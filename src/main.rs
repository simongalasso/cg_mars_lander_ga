extern crate glutin_window;
extern crate opengl_graphics;
extern crate piston;

use piston::event_loop::*;
use piston::input::*;

mod display;
mod maths;
mod ga;

use crate::display::display::*;
use crate::maths::do_intersect::*;
use crate::ga::ga::*;

/* --------------------------------------------------------- */
/* -   SHIP   ---------------------------------------------- */
/* --------------------------------------------------------- */

pub struct Ship {
    pos: Pos,
    angle: f64,
    power: f64,
    h_speed: f64,
    v_speed: f64,
    fuel: f64,
    is_dead: bool,
    is_solution: bool,
    crash_pos: Pos,
    path: Vec<Pos>
}

impl Ship {
    fn new() -> Self {
        return Self {
            pos: Pos::from(2500.0, 2700.0),
            angle: 0.0,
            power: 0.0,
            h_speed: 0.0,
            v_speed: 0.0,
            fuel: 550.0,
            is_dead: false,
            is_solution: false,
            crash_pos: Pos::from(0.0, 0.0),
            path: vec![]
        }
    }

    fn is_out_of_map(&self) -> bool {
        return self.pos.x < 0.0 || self.pos.x >= 7000.0 || self.pos.y < 0.0 || self.pos.y >= 3000.0;
    }

    fn simulate(&mut self, angle: f64, power: f64, gravity: f64) {
        self.angle += match angle {
            target_angle if target_angle > 0.0 => (target_angle as f64 - self.angle).min(15.0),
            target_angle => (target_angle - self.angle).max(-15.0)
        };
        self.power += match power {
            target_thrust if target_thrust > 0.0 => (target_thrust as f64 - self.power).min(1.0),
            target_thrust => (target_thrust - self.power).max(-1.0)
        };
        self.fuel -= self.power;

        let v_acc = (self.power * (self.angle.to_radians()).cos()) - gravity;
        // eprintln!("v_acc: {}", v_acc);
        self.pos.y = self.pos.y + self.v_speed + 0.5 * v_acc;
        self.v_speed += v_acc;

        let h_acc = self.power * (-self.angle.to_radians()).sin();
        // eprintln!("h_acc: {}", h_acc);
        self.pos.x = self.pos.x + self.h_speed + 0.5 * h_acc;
        self.h_speed += h_acc;

        self.path.push(self.pos.clone());
    }
}

/* --------------------------------------------------------- */
/* -   GAME   ---------------------------------------------- */
/* --------------------------------------------------------- */

const MAX_TURNS: i32 = 60;
const POPULATION_COUNT: i32 = 100;
const RAYS_MODE: i32 = 0;
const SHIPS_MODE: i32 = 1;

struct Game {
    display_mode: i32,
    gravity: f64,
    map: Vec<Pos>,
    landing_zone_xmin: f64,
    landing_zone_xmax: f64,
    landing_zone_y: f64,
    population: Population,
    ships: Vec<Ship>,
    turn: i32,
    paused: bool,
    next_turn: bool
}

impl Game {
    fn setup() -> Self {
        return Self {
            display_mode: RAYS_MODE,
            gravity: 3.711,
            map: vec![
                Pos::from(0.0, 100.0),
                Pos::from(1000.0, 500.0),
                Pos::from(1500.0, 1500.0),
                Pos::from(3000.0, 1000.0),
                Pos::from(4000.0, 150.0),
                Pos::from(5500.0, 150.0),
                Pos::from(6999.0, 800.0)
            ],
            landing_zone_xmin: 4000.0,
            landing_zone_xmax: 5500.0,
            landing_zone_y: 150.0,
            population: Population::new(POPULATION_COUNT),
            ships: (0..POPULATION_COUNT).map(|_| Ship::new()).collect::<Vec<Ship>>(),
            turn: 0,
            paused: true,
            next_turn: false
        }
    }
}

/* --------------------------------------------------------- */
/* -   MAIN   ---------------------------------------------- */
/* --------------------------------------------------------- */

fn main() {
    let mut display: Display = Display::setup(7000.0 * SCREEN_SCALE, 3000.0 * SCREEN_SCALE);
    let mut game: Game = Game::setup();

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut display.window) {
        if let Some(args) = e.press_args() {
            match args {
                Button::Keyboard(Key::Return) => {
                    game.display_mode = if game.display_mode == SHIPS_MODE { RAYS_MODE } else { SHIPS_MODE };
                },
                Button::Keyboard(Key::Space) => {
                    game.paused = !game.paused;
                },
                Button::Keyboard(Key::Right) => if game.paused {
                    game.next_turn = true;
                },
                // Button::Keyboard(Key::Left) => {
                // }
                _ => {}
            }
        }
        match game.display_mode {
            RAYS_MODE => {
                if !game.paused || game.next_turn {
                    if let Some(_event) = e.update_args() {
                        game.turn = 0;
                        game.ships.clear();
                        for _ in 0..game.population.entities.len() {
                            game.ships.push(Ship::new());
                        }
                        while game.turn < MAX_TURNS && game.ships.iter().filter(|ship| !ship.is_dead).count() > 0 {
                            for (i, entity) in game.population.entities.iter_mut().enumerate() {
                                if !game.ships[i].is_dead {
                                    let instruction = entity.genes[game.turn as usize];
                                    let angle: f64 = instruction.0 as f64;
                                    let power: f64 = instruction.1 as f64;
                                    let prev_pos = game.ships[i].pos.clone();
                                    game.ships[i].simulate(angle, power, game.gravity);
                                    for index in 0..(game.map.len() - 1) {
                                        let a = game.map[index].clone();
                                        let b = game.map[index + 1].clone();
                                        if do_intersect(&a, &b, &prev_pos, &game.ships[i].pos)
                                            || game.ships[i].is_out_of_map()
                                            || game.ships[i].fuel == 0.0
                                        {
                                            game.ships[i].crash_pos = game.ships[i].pos.clone();
                                            if game.ships[i].pos.x > game.landing_zone_xmin && game.ships[i].pos.x < game.landing_zone_xmax && game.ships[i].angle == 0.0 && game.ships[i].v_speed > -40.0 && game.ships[i].h_speed.abs() <= 20.0 {
                                                game.ships[i].is_solution = true;
                                            }
                                            game.ships[i].is_dead = true;
                                            entity.calc_fitness(&game.ships[i], game.landing_zone_xmax, game.landing_zone_xmin);
                                        }
                                    }
                                }
                            }
                            game.turn += 1;
                            game.next_turn = false;
                        }
                        game.population.evaluate();
                        game.population.generate();
                    }
                }
                if let Some(event) = e.render_args() {
                    display.gl.draw(event.viewport(), |_context, gl| {
                        graphics::clear(GREY1, gl);
                    });
                    display.render_ground(&event, &game.map);
                    for ship in game.ships.iter() {
                        display.render_ray(&event, &ship);
                    }
                }
            },
            SHIPS_MODE => {
                if let Some(_event) = e.update_args() {
                    if !game.paused || game.next_turn {
                        if game.turn == MAX_TURNS || game.ships.iter().filter(|ship| !ship.is_dead).count() == 0 {
                            game.ships.clear();
                            game.population.evaluate();
                            game.population.generate();
                            for _ in 0..game.population.entities.len() {
                                game.ships.push(Ship::new());
                            }
                            game.turn = 0;
                        } else {
                            for (i, entity) in game.population.entities.iter_mut().enumerate() {
                                if !game.ships[i].is_dead {
                                    let instruction = entity.genes[game.turn as usize];
                                    let angle: f64 = instruction.0 as f64;
                                    let power: f64 = instruction.1 as f64;
                                    let prev_pos = game.ships[i].pos.clone();
                                    game.ships[i].simulate(angle, power, game.gravity);
                                    for index in 0..(game.map.len() - 1) {
                                        let a = game.map[index].clone();
                                        let b = game.map[index + 1].clone();
                                        if do_intersect(&a, &b, &prev_pos, &game.ships[i].pos)
                                            || game.ships[i].is_out_of_map()
                                            || game.ships[i].fuel == 0.0
                                        {
                                            game.ships[i].crash_pos = game.ships[i].pos.clone();
                                            if game.ships[i].pos.x > game.landing_zone_xmin && game.ships[i].pos.x < game.landing_zone_xmax && game.ships[i].angle == 0.0 && game.ships[i].v_speed > -40.0 && game.ships[i].h_speed.abs() <= 20.0 {
                                                game.ships[i].is_solution = true;
                                            }
                                            game.ships[i].is_dead = true;
                                            entity.calc_fitness(&game.ships[i], game.landing_zone_xmax, game.landing_zone_xmin);
                                        }
                                    }
                                }
            
                            }
                        }
                        game.turn += 1;
                        game.next_turn = false;
                    }
                }
                if let Some(event) = e.render_args() {
                    display.gl.draw(event.viewport(), |_context, gl| {
                        graphics::clear(GREY1, gl);
                    });
                    display.render_ground(&event, &game.map);
                    for ship in game.ships.iter() {
                        if !ship.is_dead || ship.is_solution {
                            display.render_ship(&event, &ship.pos, ship.angle, ship.power, ship.is_solution);
                        }
                    }
                }
            },
            _ => {}
        }
    }
}