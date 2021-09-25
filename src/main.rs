extern crate glutin_window;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use piston::event_loop::*;
use piston::input::*;
use rand::Rng;

mod display;
mod maths;

use crate::display::display::*;
use crate::maths::do_intersect::*;

/* --------------------------------------------------------- */
/* -   CHROMOSOME   ---------------------------------------- */
/* --------------------------------------------------------- */

#[derive(Debug, Clone, PartialEq)]
struct Chromosome {
    genes: Vec<(i32, i32)>, // angle, thrust
    fitness: f64,
    prob: f64
}

impl Chromosome {
    fn new() -> Self {
        let mut genes = vec![];
        for _ in 0..CHROMOSOME_SIZE {
            let rand_angle = rand::thread_rng().gen_range(-15..=15);
            let rand_power = rand::thread_rng().gen_range(-1..=1);
            genes.push((rand_angle, rand_power));
        }
        return Self {
            genes: genes,
            fitness: 0.0,
            prob: 0.0
        };
    }
}

/* --------------------------------------------------------- */
/* -   SHIP   ---------------------------------------------- */
/* --------------------------------------------------------- */

#[derive(Clone, PartialEq)]
pub struct Ship {
    chromosome: Chromosome,
    pos: Pos,
    angle: f64,
    power: f64,
    h_speed: f64,
    v_speed: f64,
    fuel: f64,
    is_dead: bool,
    is_solution: bool,
    crash_pos: Pos,
    path: Vec<Pos>,
    is_best: bool,
    is_elite: bool,
    crash_zone_index: usize
}

impl Ship {
    fn new() -> Self {
        return Self {
            chromosome: Chromosome::new(),
            pos: Pos::from(6500.0, 2000.0),
            angle: 0.0,
            power: 0.0,
            h_speed: 0.0,
            v_speed: 0.0,
            fuel: 1200.0,
            is_dead: false,
            is_solution: false,
            crash_pos: Pos::from(0.0, 0.0),
            path: vec![],
            is_best: false,
            is_elite: false,
            crash_zone_index: 0
        }
    }

    fn is_out_of_map(&self) -> bool {
        return self.pos.x < 0.0 || self.pos.x >= 7000.0 || self.pos.y < 0.0 || self.pos.y >= 3000.0;
    }

    fn simulate(&mut self, angle: f64, power: f64, gravity: f64) {
        let clamped_angle = angle.max(-15.0).min(15.0);
        self.angle += clamped_angle;
        self.angle = self.angle.max(-90.0).min(90.0);
        let clamped_power = power.max(-1.0).min(1.0);
        self.power += clamped_power;
        self.power = self.power.max(0.0).min(4.0);
        self.fuel -= self.power;
        let v_acc = (self.power * (self.angle.to_radians()).cos()) - gravity;
        self.pos.y = self.pos.y + self.v_speed + 0.5 * v_acc;
        self.v_speed += v_acc;
        let h_acc = self.power * (-self.angle.to_radians()).sin();
        self.pos.x = self.pos.x + self.h_speed + 0.5 * h_acc;
        self.h_speed += h_acc;
        self.path.push(self.pos.clone());
    }

    fn crossover(&self, partner: Ship) -> [Ship; 2] {
        let mut childs: [Ship; 2] = [Ship::new(), Ship::new()];

        for i in 0..CHROMOSOME_SIZE {
            let gene_a_angle = self.chromosome.genes[i].0 as f64;
            let gene_a_power = self.chromosome.genes[i].1 as f64;
            let gene_b_angle = partner.chromosome.genes[i].0 as f64;
            let gene_b_power = partner.chromosome.genes[i].1 as f64;
            let r: f64 = rand::thread_rng().gen_range(0.0..=1.0);

            childs[0].chromosome.genes[i as usize] = ((r * gene_a_angle + (1.0 - r) * gene_b_angle).round() as i32, (r * gene_a_power + (1.0 - r) * gene_b_power).round()  as i32);
            childs[1].chromosome.genes[i as usize] = (((1.0 - r) * gene_a_angle + r * gene_b_angle).round()  as i32, ((1.0 - r) * gene_a_power + r * gene_b_power).round()  as i32);
        }
        return childs;
    }

    fn mutate(&mut self, mutation_rate: f64) {
        for i in 0..self.chromosome.genes.len() {
            if rand::thread_rng().gen_range(0.0..=1.0) < mutation_rate {
                let rand_angle = rand::thread_rng().gen_range(-15..=15);
                let rand_power = rand::thread_rng().gen_range(-1..=1);
                self.chromosome.genes[i] = (rand_angle, rand_power);
            }
        }
    }
}

/* --------------------------------------------------------- */
/* -   GAME   ---------------------------------------------- */
/* --------------------------------------------------------- */

const POPULATION_COUNT: usize = 100;
const CHROMOSOME_SIZE: usize = 500;
const RAYS_MODE: i32 = 0;
const SHIPS_MODE: i32 = 1;

struct Game {
    display_mode: i32,
    gravity: f64,
    map: Vec<Pos>,
    landing_zone_xmin: f64,
    landing_zone_xmax: f64,
    landing_zone_y: f64,
    ships: Vec<Ship>,
    turn: i32,
    paused: bool,
    next_turn: bool,
    mutation_rate: f64,
    generation: i32
}

impl Game {
    fn setup() -> Self {
        return Self {
            display_mode: RAYS_MODE,
            gravity: 3.711,
            map: vec![
                Pos::from(0.0, 1800.0),
                Pos::from(300.0, 1200.0),
                Pos::from(1000.0, 1550.0),
                Pos::from(2000.0, 1200.0),
                Pos::from(2500.0, 1650.0),
                Pos::from(3700.0, 220.0),
                Pos::from(4700.0, 220.0),
                Pos::from(4750.0, 1000.0),
                Pos::from(4700.0, 1650.0),
                Pos::from(4000.0, 1700.0),
                Pos::from(3700.0, 1600.0),
                Pos::from(3750.0, 1900.0),
                Pos::from(4000.0, 2100.0),
                Pos::from(4900.0, 2050.0),
                Pos::from(5100.0, 1000.0),
                Pos::from(5500.0, 500.0),
                Pos::from(6200.0, 800.0),
                Pos::from(6999.0, 600.0)
            ],
            landing_zone_xmin: 3700.0,
            landing_zone_xmax: 4700.0,
            landing_zone_y: 220.0,
            ships: (0..POPULATION_COUNT).map(|_| Ship::new()).collect::<Vec<Ship>>(),
            turn: 0,
            paused: true,
            next_turn: false,
            mutation_rate: 0.01,
            generation: 0
        }
    }
    
    fn pick_partner(&mut self) -> Ship {
        let r: f64 = rand::thread_rng().gen_range(0.0..=1.0);
        if r < self.ships[0].chromosome.prob {
            return self.ships[0].clone();
        }
        let mut selected: Option<Ship> = None;
        for i in 1..POPULATION_COUNT {
            if self.ships[i - 1].chromosome.prob < r && r <= self.ships[i].chromosome.prob {
                selected = Some(self.ships[i].clone());
                break;
            }
        }
        return selected.unwrap();
    }

    fn get_elites(&self) -> Vec<Ship> {
        let mut ships = self.ships.clone();
        let mut elites = vec![];
        ships.sort_by(|a, b| b.chromosome.fitness.partial_cmp(&a.chromosome.fitness).unwrap());
        for i in 0..(POPULATION_COUNT as f64 * 0.2) as usize {
            elites.push(ships[i].clone());
        }
        return elites;
    }
    
    fn generate(&mut self) {
        let mut new_ships: Vec<Ship> = vec![];
        let mut fitness_sum: f64 = 0.0;
        for i in 0..POPULATION_COUNT {
            fitness_sum += self.ships[i].chromosome.fitness;
        }
        let mut prob_sum = 0.0;
        for i in 0..POPULATION_COUNT {
            self.ships[i].chromosome.prob = prob_sum + self.ships[i].chromosome.fitness / fitness_sum;
            prob_sum += self.ships[i].chromosome.prob;
        }
    
        for _ in 0..(POPULATION_COUNT / 2) {
            let partner_a = self.pick_partner();
            let mut partner_b = self.pick_partner();
            while partner_a == partner_b {
                partner_b = self.pick_partner();
            }
            let mut childs: [Ship; 2] = partner_a.crossover(partner_b);
            childs[0].mutate(self.mutation_rate);
            childs[1].mutate(self.mutation_rate);
            new_ships.push(childs[0].clone());
            new_ships.push(childs[1].clone());
        }
        self.ships = new_ships;
        self.generation += 1;
    }

    fn calc_min_dist(&self, crash_pos: &Pos, crash_zone_index: i32) -> f64 {
        let mut landing_zone_index = 0;
        for i in 0..(self.map.len() - 1) {
            if self.map[i].x == self.landing_zone_xmin && self.map[i + 1].x == self.landing_zone_xmax && self.map[i].y == self.landing_zone_y {
                landing_zone_index = i as i32;
                break;
            }
        }

        if crash_zone_index == landing_zone_index {
            return 0.0;
        }

        let mut dir: i32 = match crash_zone_index > landing_zone_index {
            true => -1, // crashed on right of landing zone
            _ => 1// crashed on right of landing zone
        };

        let offset = match dir { // position on the crash_zone
            -1 => { // going left
                let dist_x = crash_pos.x - self.map[crash_zone_index as usize].x;
                let dist_y = crash_pos.y - self.map[crash_zone_index as usize].y;
                ((dist_x * dist_x) + (dist_y * dist_y)).sqrt()
            },
            _ => { // going right
                let dist_x = crash_pos.x - self.map[crash_zone_index as usize + 1].x;
                let dist_y = crash_pos.y - self.map[crash_zone_index as usize + 1].y;
                ((dist_x * dist_x) + (dist_y * dist_y)).sqrt()
            }
        };

        let mut dist = 0.0;
        let mut prev = crash_zone_index;
        let mut index = crash_zone_index + dir;
        while index >= 0 && index < self.map.len() as i32 && index != landing_zone_index {
            let dist_x = self.map[prev as usize].x - self.map[index as usize].x;
            let dist_y = self.map[prev as usize].y - self.map[index as usize].y;
            dist += ((dist_x * dist_x) + (dist_y * dist_y)).sqrt();
            // eprintln!("{}", dist);
            prev = index;
            index += dir;
        }
        // eprintln!("---");
        // eprintln!("{} + {} = {}", dist, offset, dist + offset);
        return dist + offset;
    }

    fn calc_fitness(&mut self, ship_index: usize) {
        if self.ships[ship_index].crash_pos.x < 0.0 || self.ships[ship_index].crash_pos.x >= 7000.0 || self.ships[ship_index].crash_pos.y < 0.0 || self.ships[ship_index].crash_pos.y >= 7000.0 {
            self.ships[ship_index].chromosome.fitness = 1.0;
            return;
        }
        let speed = ((self.ships[ship_index].h_speed * self.ships[ship_index].h_speed) + (self.ships[ship_index].v_speed * self.ships[ship_index].v_speed)).sqrt();
        if self.ships[ship_index].crash_pos.x < self.landing_zone_xmin || self.ships[ship_index].crash_pos.x > self.landing_zone_xmax || self.ships[ship_index].crash_pos.y > self.landing_zone_y {
            let dist = self.calc_min_dist(&self.ships[ship_index].crash_pos, self.ships[ship_index].crash_zone_index as i32) / ((7000.0 * 7000.0) as f64 + (3000.0 * 3000.0) as f64).sqrt(); // FIXME, can be longer (divide by the max dist of the generation)
            let mut score = 100.0 - (dist * 100.0);
            let speed_score: f64 = 0.1 * (speed - 100.0).max(0.0);
		    score -= speed_score;
            self.ships[ship_index].chromosome.fitness = score; // 0 to 100
        } else if !self.ships[ship_index].is_solution {
            let mut x_score = 0.0;
            if 20.0 < (self.ships[ship_index].h_speed).abs() {
                x_score = ((self.ships[ship_index].h_speed).abs() - 20.0) / 2.0;
            }
            let mut y_score = 0.0;
            if self.ships[ship_index].v_speed < -40.0 {
                y_score = (-40.0 - self.ships[ship_index].v_speed) / 2.0;
            }
            self.ships[ship_index].chromosome.fitness = 200.0 - x_score - y_score; // 100 to 200
        } else {
            self.ships[ship_index].chromosome.fitness = 200.0 + (100.0 * self.ships[ship_index].fuel / 550.0); // 200 to 300
        }
    }
    
    fn evaluate(&mut self) -> usize {
        let mut max_fitness: f64 = 0.0;
        let mut total_fitness: f64 = 0.0;
        let mut best = 0;
    
        for i in 0..POPULATION_COUNT {
            self.calc_fitness(i);
            if self.ships[i].chromosome.fitness > max_fitness {
                max_fitness = self.ships[i].chromosome.fitness;
                best = i;
            }
            total_fitness += self.ships[i].chromosome.fitness;
        }
        let fitness_average: i32 = (total_fitness / POPULATION_COUNT as f64) as i32;
        println!("gen: {} | av: {} | max: {}", self.generation, fitness_average, max_fitness as i32);
        return best;
    }
}

/* --------------------------------------------------------- */
/* -   MAIN   ---------------------------------------------- */
/* --------------------------------------------------------- */

fn main() {
    let mut display: Display = Display::setup(7000.0 * SCREEN_SCALE, 3000.0 * SCREEN_SCALE);
    let mut game: Game = Game::setup();
    eprintln!("{:?}", game.ships[0].chromosome.genes);

    let mut to_display = vec![];
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
                if let Some(_event) = e.update_args() {
                    if !game.paused || game.next_turn {
                        to_display = vec![];
                        game.turn = 0;
                        while game.turn < CHROMOSOME_SIZE as i32 && game.ships.iter().filter(|ship| !ship.is_dead).count() > 0 {
                            for ship in game.ships.iter_mut() {
                                if !ship.is_dead {
                                    let instruction = ship.chromosome.genes[game.turn as usize];
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
                                            if ship.pos.x > game.landing_zone_xmin && ship.pos.x < game.landing_zone_xmax && ship.angle == 0.0 && ship.v_speed > -40.0 && ship.h_speed.abs() <= 20.0 {
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
                        let best = game.evaluate();
                        game.ships[best].is_best = true;
                        let elites = game.get_elites();
                        for ship in game.ships.iter_mut() {
                            if elites.contains(ship) {
                                ship.is_elite = true;
                            }
                        }
                        to_display = game.ships.clone();
                        game.generate();
                        for i in 0..elites.len() {
                            game.ships[i] = elites[i].clone();
                            game.ships[i].is_elite = true;
                        }
                    }
                }
                if let Some(event) = e.render_args() {
                    display.gl.draw(event.viewport(), |_context, gl| {
                        graphics::clear(GREY1, gl);
                    });
                    display.render_ground(&event, &game.map);
                    for ship in to_display.iter() {
                        display.render_ray(&event, &ship);
                    }
                }
            },
            // SHIPS_MODE => {
            //     if let Some(_event) = e.update_args() {
            //         if !game.paused || game.next_turn {
            //             if game.turn == CHROMOSOME_SIZE as i32 || game.ships.iter().filter(|ship| !ship.is_dead).count() == 0 {
            //                 let best = game.evaluate();
            //                 game.ships[best].is_best = true;
            //                 let elites = game.get_elites();
            //                 for ship in game.ships.iter_mut() {
            //                     if elites.contains(ship) {
            //                         ship.is_elite = true;
            //                     }
            //                 }
            //                 to_display = game.ships.clone();
            //                 game.generate();
            //                 for i in 0..elites.len() {
            //                     game.ships[i] = elites[i].clone();
            //                     game.ships[i].is_elite = true;
            //                 }
            //                 game.turn = 0;
            //             } else {
            //                 for ship in game.ships.iter_mut() {
            //                     if !ship.is_dead {
            //                         let instruction = ship.chromosome.genes[game.turn as usize];
            //                         let angle: f64 = instruction.0 as f64;
            //                         let power: f64 = instruction.1 as f64;
            //                         let prev_pos = ship.pos.clone();
            //                         ship.simulate(angle, power, game.gravity);
            //                         for index in 0..(game.map.len() - 1) {
            //                             let a = game.map[index].clone();
            //                             let b = game.map[index + 1].clone();
            //                             if do_intersect(&a, &b, &prev_pos, &ship.pos)
            //                                 || ship.is_out_of_map()
            //                                 || ship.fuel == 0.0
            //                             {
            //                                 ship.crash_pos = ship.pos.clone();
            //                                 if ship.pos.x > game.landing_zone_xmin && ship.pos.x < game.landing_zone_xmax && ship.angle == 0.0 && ship.v_speed > -40.0 && ship.h_speed.abs() <= 20.0 {
            //                                     ship.is_solution = true;
            //                                 }
            //                                 ship.is_dead = true;
            //                             }
            //                         }
            //                     }
            
            //                 }
            //             }
            //             game.turn += 1;
            //             game.next_turn = false;
            //         }
            //     }
            //     if let Some(event) = e.render_args() {
            //         display.gl.draw(event.viewport(), |_context, gl| {
            //             graphics::clear(GREY1, gl);
            //         });
            //         display.render_ground(&event, &game.map);
            //         for ship in game.ships.iter() {
            //             if !ship.is_dead || ship.is_solution {
            //                 display.render_ship(&event, &ship.pos, ship.angle, ship.power, ship.is_solution);
            //             }
            //         }
            //     }
            // },
            _ => {}
        }
    }
}