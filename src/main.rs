extern crate glutin_window;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use std::time::{Instant};
use piston::event_loop::*;
use piston::input::*;

mod display;
mod maths;
mod game;

use crate::display::display::*;
use crate::display::args::*;
use crate::game::game::*;
use crate::maths::utils::*;

fn run_genetic(game: &mut Game) {
    let best = game.evaluate();
    game.ships[best].is_best = true;
    let elites = game.get_elites();
    for ship in game.ships.iter_mut() {
        if elites.contains(ship) {
            ship.is_elite = true;
        }
    }
    game.generate();
    for i in 0..elites.len() {
        game.ships[i] = elites[i].clone();
        game.ships[i].is_elite = true;
    }
    game.turn = 0;
    while game.turn < CHROMOSOME_SIZE && game.ships.iter().filter(|ship| !ship.is_dead).count() > 0 {
        for ship in game.ships.iter_mut() {
            if !ship.is_dead {
                let instruction = ship.chromosome.genes[game.turn];
                let angle: f64 = instruction.0 as f64;
                let power: f64 = instruction.1 as f64;
                let prev_pos = ship.pos.clone();
                ship.simulate(angle, power, game.gravity);
                for index in 0..(game.map.len() - 1) {
                    let a = game.map[index].clone();
                    let b = game.map[index + 1].clone();
                    if do_intersect(&a, &b, &prev_pos, &ship.pos)
                        || ship.is_out_of_map()
                        || ship.fuel == 0.0
                    {
                        ship.crash_pos = ship.pos.clone();
                        ship.crash_zone_index = index;
                        if a.x == game.landing_zone_xmin && b.x == game.landing_zone_xmax && a.y == game.landing_zone_y && b.y == game.landing_zone_y
                            && ship.angle == 0.0 && ship.v_speed > -40.0 && ship.h_speed.abs() <= 20.0
                        {
                            if game.best_ship.is_none() || ship.chromosome.fitness > game.best_ship.as_ref().unwrap().chromosome.fitness {
                                game.best_ship = Some(ship.clone());
                            }
                            ship.is_solution = true;
                        }
                        ship.is_dead = true;
                    }
                }
            }
        }
        game.turn += 1;
        game.next_turn = false;
    }
}

fn main() {
    let mut display: Display = Display::setup(7000.0 * SCREEN_SCALE, 3000.0 * SCREEN_SCALE);
    let mut game: Game = Game::setup();

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
                    if duration > 900 {
                        game.search_ended = true;
                        game.paused = true;
                        game.turn = 0;
                        eprintln!("generations: {}", game.generation)
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
            if game.display_mode == RAYS_MODE {
                if !game.search_ended {
                    for ship in game.ships.iter() {
                        display.render_ray(&event, &ship);
                    }
                } else {
                    let best_ship: &Ship = game.best_ship.as_ref().unwrap();
                    if game.turn < best_ship.path.len() {
                        display.render_ship(&event, &best_ship.path[game.turn], best_ship.chromosome.genes[game.turn].0 as f64, best_ship.chromosome.genes[game.turn].1 as f64);
                    } else {
                        game.turn = 0;
                    }
                }
            }
        }
    }
}