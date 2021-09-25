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
    fn new(length: usize) -> Self {
        let mut genes = vec![];
        for i in 0..CHROMOSOME_SIZE {
            let mut rand_angle = rand::thread_rng().gen_range(-15..=15);
            let mut rand_power = rand::thread_rng().gen_range(-1..=1);
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
    is_elite: bool
}

impl Ship {
    fn new() -> Self {
        return Self {
            chromosome: Chromosome::new(CHROMOSOME_SIZE as usize),
            pos: Pos::from(2600.0, 2800.0),
            angle: 0.0,
            power: 0.0,
            h_speed: 0.0,
            v_speed: 0.0,
            fuel: 550.0,
            is_dead: false,
            is_solution: false,
            crash_pos: Pos::from(0.0, 0.0),
            path: vec![],
            is_best: false,
            is_elite: false
        }
    }

    fn is_out_of_map(&self) -> bool {
        return self.pos.x < 0.0 || self.pos.x >= 7000.0 || self.pos.y < 0.0 || self.pos.y >= 3000.0;
    }

    fn simulate(&mut self, angle: f64, power: f64, gravity: f64) {
        // self.angle += match angle {
        //     target_angle if target_angle > 0.0 => (target_angle as f64 - self.angle).min(15.0),
        //     target_angle => (target_angle - self.angle).max(-15.0)
        // };
        let mut clamped_angle = angle.max(-15.0).min(15.0);
        self.angle += clamped_angle;
        if self.angle > 90.0 {
            self.angle = 90.0;
        }
        if self.angle < -90.0 {
            self.angle = -90.0 ;
        }
        // self.power += match power {
        //     target_thrust if target_thrust > 0.0 => (target_thrust as f64 - self.power).min(1.0),
        //     target_thrust => (target_thrust - self.power).max(-1.0)
        // };
        let mut clamped_power = power.max(-1.0).min(1.0);
        self.power += clamped_power;
        if self.power > 4.0 {
            self.power = 4.0;
        }
        if self.power < 0.0 {
            self.power = 0.0 ;
        }
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
                let mut rand_angle = rand::thread_rng().gen_range(-15..=15);
                let mut rand_power = rand::thread_rng().gen_range(-1..=1);
                self.chromosome.genes[i] = (rand_angle, rand_power);
            }
        }
    }

    pub fn calc_fitness(&mut self, lz_xmin: f64, lz_xmax: f64, lz_y: f64) {
        if self.crash_pos.x < 0.0 || self.crash_pos.x >= 7000.0 || self.crash_pos.y < 0.0 || self.crash_pos.y >= 7000.0 {
            self.chromosome.fitness = 1.0;
            return;
        }
        let speed = ((self.h_speed * self.h_speed) + (self.v_speed * self.v_speed)).sqrt();
        if self.crash_pos.x < lz_xmin || self.crash_pos.x > lz_xmax {
            let norm_dist_x = (self.crash_pos.x - (lz_xmax + lz_xmin) / 2.0).abs() / 7000.0; // 0.0 to 1.0
            let norm_dist_y = (self.crash_pos.y - lz_y).abs() / 3000.0; // 0.0 to 1.0
            let norm_dist = (norm_dist_x + norm_dist_y) / 2.0;
            let mut score = 100.0 - (norm_dist * 100.0);
            let speed_score: f64 = 0.1 * (speed - 100.0).max(0.0);
		    score -= speed_score;
            self.chromosome.fitness = score; // 0 to 100
        } else if !self.is_solution {
            let mut x_score = 0.0;
            if 20.0 < (self.h_speed).abs() {
                x_score = ((self.h_speed).abs() - 20.0) / 2.0;
            }
            let mut y_score = 0.0;
            if self.v_speed < -40.0 {
                y_score = (-40.0 - self.v_speed) / 2.0;
            }
            self.chromosome.fitness = 200.0 - x_score - y_score; // 100 to 200
        } else {
            self.chromosome.fitness = 200.0 + (100.0 * self.fuel / 550.0); // 200 to 300
        }
    }
}

/* --------------------------------------------------------- */
/* -   GAME   ---------------------------------------------- */
/* --------------------------------------------------------- */

const POPULATION_COUNT: usize = 100;
const CHROMOSOME_SIZE: usize = 100;
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
                Pos::from(0.0, 1000.0),
                Pos::from(300.0, 1500.0),
                Pos::from(350.0, 1400.0),
                Pos::from(500.0, 2000.0),
                Pos::from(800.0, 1800.0),
                Pos::from(1000.0, 2500.0),
                Pos::from(1200.0, 2100.0),
                Pos::from(1500.0, 2400.0),
                Pos::from(2000.0, 1000.0),
                Pos::from(2200.0, 500.0),
                Pos::from(2500.0, 100.0),
                Pos::from(2900.0, 800.0),
                Pos::from(3000.0, 500.0),
                Pos::from(3200.0, 1000.0),
                Pos::from(3500.0, 2000.0),
                Pos::from(3800.0, 800.0),
                Pos::from(4000.0, 200.0),
                Pos::from(5000.0, 200.0),
                Pos::from(5500.0, 1500.0),
                Pos::from(6999.0, 2800.0)
            ],
            landing_zone_xmin: 4000.0,
            landing_zone_xmax: 5000.0,
            landing_zone_y: 200.0,
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
    
    fn evaluate(&mut self) -> usize {
        let mut max_fitness: f64 = 0.0;
        let mut total_fitness: f64 = 0.0;
        let mut best = 0;
    
        for i in 0..POPULATION_COUNT {
            self.ships[i].calc_fitness(self.landing_zone_xmin, self.landing_zone_xmax, self.landing_zone_y);
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
            //             if game.turn == CHROMOSOME_SIZE || game.ships.iter().filter(|ship| !ship.is_dead).count() == 0 {
            //                 game.ships.clear();
            //                 let best = game.population.evaluate();
            //                 game.ships[best].is_best = true;
            //                 game.population.generate();
            //                 for _ in 0..game.population.chromosomes.len() {
            //                     game.ships.push(Ship::new());
            //                 }
            //                 game.turn = 0;
            //             } else {
            //                 for (i, chromosome) in game.population.chromosomes.iter_mut().enumerate() {
            //                     if !game.ships[i].is_dead {
            //                         let instruction = chromosome.genes[game.turn as usize];
            //                         let angle: f64 = instruction.0 as f64;
            //                         let power: f64 = instruction.1 as f64;
            //                         let prev_pos = game.ships[i].pos.clone();
            //                         game.ships[i].simulate(angle, power, game.gravity);
            //                         for index in 0..(game.map.len() - 1) {
            //                             let a = game.map[index].clone();
            //                             let b = game.map[index + 1].clone();
            //                             if do_intersect(&a, &b, &prev_pos, &game.ships[i].pos)
            //                                 || game.ships[i].is_out_of_map()
            //                                 || game.ships[i].fuel == 0.0
            //                             {
            //                                 game.ships[i].crash_pos = game.ships[i].pos.clone();
            //                                 if game.ships[i].pos.x > game.landing_zone_xmin && game.ships[i].pos.x < game.landing_zone_xmax && game.ships[i].angle == 0.0 && game.ships[i].v_speed > -40.0 && game.ships[i].h_speed.abs() <= 20.0 {
            //                                     game.ships[i].is_solution = true;
            //                                 }
            //                                 game.ships[i].is_dead = true;
            //                                 chromosome.calc_fitness(&game.ships[i], game.landing_zone_xmin, game.landing_zone_xmax, game.landing_zone_y);
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