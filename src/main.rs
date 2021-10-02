extern crate glutin_window;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use std::time::{Instant};
use piston::event_loop::*;
use piston::input::*;

mod parsing;
mod display;
mod maths;
mod game;

use parsing::args::{Config};
use parsing::parser::{parse_file};
use display::display::*;
use display::args::*;
use game::game::*;
use maths::utils::*;

fn run_genetic(game: &mut Game) {
    game.turn = 0;
    while game.turn < CHROMOSOME_SIZE && game.ships.iter().filter(|ship| !ship.is_dead).count() > 0 {
        // eprintln!("ok");
        for ship in game.ships.iter_mut().filter(|ship| !ship.is_dead) {
            let instruction = ship.chromosome.genes[game.turn];
            let angle: f32 = instruction.0 as f32;
            let power: f32 = instruction.1 as f32;
            let prev_pos = ship.pos.clone();
            ship.simulate(angle, power, game.gravity);
            for index in 0..(game.map.len() - 1) {
                // eprintln!("index: {}", index);
                let a = game.map[index].clone();
                let b = game.map[index + 1].clone();

                if do_intersect(&a, &b, &prev_pos, &ship.pos) {
                    // eprintln!("A");
                    ship.crash_pos = find_intersection_point(&a, &b, &prev_pos, &ship.pos);
                    ship.crash_zone_index = index;
                    if ship.crash_zone_index == game.landing_zone_index && ship.angle == 0.0 && ship.v_speed >= -40.0 && ship.h_speed.abs() <= 20.0
                    {
                        if game.best_ship.is_none() || ship.chromosome.fitness > game.best_ship.as_ref().unwrap().chromosome.fitness {
                            game.best_ship = Some(ship.clone());
                        }
                        ship.is_solution = true;
                    }
                    ship.is_dead = true;
                    break;
                }
            }
            if !ship.is_dead && ship.is_out_of_map() { // FIXME, it mean no intersection and out of map, maybe find better way to do that
                // eprintln!("B");
                ship.is_out = true;
                ship.is_dead = true;
            }
        }
        game.turn += 1;
        game.next_turn = false;
    }
    game.evaluate();
    // for ship in game.ships.iter() {
    //     eprintln!("{}, is_out: {}", ship.chromosome.fitness, ship.is_out);
    // }
    let elites = game.get_elites();
    game.previous_population = game.ships.clone();
    game.generate();
    for i in 0..elites.len() {
        game.ships[i] = elites[i].clone();
        game.ships[i].is_elite = true;
    }
}

fn main() {
    let config: Config = Config::new();
    match parse_file(&config.level_file) {
        Ok(level_data) => {
            let mut display: Display = Display::setup(7000.0 * SCREEN_SCALE, 3000.0 * SCREEN_SCALE);
            let mut game: Game = Game::setup(&level_data);
            
            eprintln!("pos: {:?}", game.level_data.pos);
            eprintln!("angle: {}", game.level_data.angle);
            eprintln!("power: {}", game.level_data.power);
            eprintln!("h_speed: {}", game.level_data.h_speed);
            eprintln!("v_speed: {}", game.level_data.v_speed);
            eprintln!("fuel: {}", game.level_data.fuel);
            eprintln!("landing_zone_xmin: {}", game.landing_zone_xmin);
            eprintln!("landing_zone_xmax: {}", game.landing_zone_xmax);
            eprintln!("landing_zone_y: {}", game.landing_zone_y);
            eprintln!("landing_zone_index: {}", game.landing_zone_index);
            eprintln!("map: {:#?}", game.map);

            let mut duration: u128 = 0;
            let mut events = Events::new(EventSettings::new());
            while let Some(e) = events.next(&mut display.window) {
                handle_args(&e, &mut game);
                if let Some(_event) = e.update_args() {
                    if !game.paused || game.next_turn {
                        if !game.search_ended {
                            let start_time = Instant::now();
                            run_genetic(&mut game);
                            duration += start_time.elapsed().as_millis();
                            if duration > config.time_limit {
                                game.search_ended = true;
                                game.paused = true;
                                game.turn = 0;
                                eprintln!("generations: {}", game.generation);
                            }
                        } else {
                            game.turn += 1;
                            game.next_turn = false;
                        }
                    }
                }
                if let Some(event) = e.render_args() {
                    display.clear_window(&event);
                    display.render_ground(&event, &game.map);
                    if !game.search_ended {
                        for ship in game.previous_population.iter() {
                            display.render_ray(&event, &ship, if ship.is_solution { GREEN } else if ship.is_elite { BLUE } else { RED });
                        }
                        match game.best_ship {
                            Some(ref ship) => display.render_ray(&event, ship, GOLD),
                            None => {}
                        }
                    } else {
                        let best_ship: &Ship = game.best_ship.as_ref().unwrap();
                        if game.turn < best_ship.path.len() {
                            let mut next_angle = game.level_data.angle as i32;
                            let mut next_power = game.level_data.power as i32;
                            for i in 0..(game.turn + 1) {
                                next_angle = (next_angle + (best_ship.chromosome.genes[i].0).min(15).max(-15)).min(90).max(-90);
                                next_power = (next_power + (best_ship.chromosome.genes[i].1).min(1).max(-1)).min(4).max(0);
                            }
                            display.render_ray(&event, &best_ship, GREEN);
                            display.render_ship(&event, &best_ship.path[game.turn], next_angle as f32, next_power as f32);
                        } else {
                            game.turn = 0;
                        }
                    }
                }
            }
        },
        Err(error) => println!("{}", error)
    }
}