extern crate rand;

use crate::maths::pos::*;
use crate::maths::utils::*;
use crate::parsing::parser::{LevelData};
use rand::prelude::*;

pub const POPULATION_COUNT: usize = 100; // default: 100
pub const CHROMOSOME_SIZE: usize = 180; // default: 200
pub const ELITE_PERCENTAGE: f32 = 0.12; // default: 0.12
pub const MUTATION_RATE: f32 = 0.01; // default: 0.01

/* --------------------------------------------------------- */
/* -   CHROMOSOME   ---------------------------------------- */
/* --------------------------------------------------------- */

#[derive(Debug, Clone, PartialEq)]
pub struct Chromosome {
    pub genes: [(i32, i32); CHROMOSOME_SIZE], // angle, thrust
    pub fitness: f32,
    pub prob: f32
}

impl Chromosome {
    fn new(rng: &mut ThreadRng) -> Self {
        let mut genes = [(0, 0); CHROMOSOME_SIZE];
        for i in 0..CHROMOSOME_SIZE {
            genes[i] = (rng.gen_range(-15..16), rng.gen_range(-1..2));
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

#[derive(Debug, Clone, PartialEq)]
pub struct Ship {
    pub chromosome: Chromosome,
    pub pos: Pos,
    pub angle: f32,
    pub power: f32,
    pub h_speed: f32,
    pub v_speed: f32,
    pub fuel: f32,
    pub is_dead: bool,
    pub is_solution: bool,
    pub crash_pos: Pos,
    pub path: Vec<Pos>,
    pub is_elite: bool,
    pub crash_zone_index: usize,
    pub is_out: bool
}

impl Ship {
    fn new(level_data: &LevelData, rng: &mut ThreadRng) -> Self {
        return Self {
            chromosome: Chromosome::new(rng),
            pos: level_data.pos.clone(),
            angle: level_data.angle,
            power: level_data.power,
            h_speed: level_data.h_speed,
            v_speed: level_data.v_speed,
            fuel: level_data.fuel,
            is_dead: false,
            is_solution: false,
            crash_pos: Pos::from(0.0, 0.0),
            path: vec![],
            is_elite: false,
            crash_zone_index: 0,
            is_out: false
        }
    }

    pub fn is_out_of_map(&self) -> bool {
        return self.pos.x < 0.0 || self.pos.x >= 7000.0 || self.pos.y < 0.0 || self.pos.y >= 3000.0;
    }

    pub fn simulate(&mut self, angle: f32, power: f32, gravity: f32) {
        let clamped_angle = angle.max(-15.0).min(15.0);
        self.angle += clamped_angle;
        self.angle = self.angle.max(-90.0).min(90.0);

        if self.fuel > 0.0 {
            let clamped_power = power.max(-1.0).min(1.0);
            self.power += clamped_power;
            self.power = self.power.max(0.0).min(4.0);
            self.fuel -= self.power;
        } else {
            self.power = 0.0;
        }
        let v_acc = (self.power * (self.angle.to_radians()).cos()) - gravity;
        self.pos.y = self.pos.y + self.v_speed + 0.5 * v_acc;
        self.v_speed += v_acc;
        let h_acc = self.power * (-self.angle.to_radians()).sin();
        self.pos.x = self.pos.x + self.h_speed + 0.5 * h_acc;
        self.h_speed += h_acc;
        self.path.push(self.pos.clone());
    }

    fn crossover(&self, partner: Ship, level_data: &LevelData, rng: &mut ThreadRng) -> [Ship; 2] {
        let mut childs: [Ship; 2] = [Ship::new(level_data, rng), Ship::new(level_data, rng)];

        for i in 0..CHROMOSOME_SIZE {
            let gene_a_angle = self.chromosome.genes[i].0 as f32;
            let gene_a_power = self.chromosome.genes[i].1 as f32;
            let gene_b_angle = partner.chromosome.genes[i].0 as f32;
            let gene_b_power = partner.chromosome.genes[i].1 as f32;
            let r: f32 = rng.gen();
            childs[0].chromosome.genes[i as usize] = ((r * gene_a_angle + (1.0 - r) * gene_b_angle).round() as i32, (r * gene_a_power + (1.0 - r) * gene_b_power).round()  as i32);
            childs[1].chromosome.genes[i as usize] = (((1.0 - r) * gene_a_angle + r * gene_b_angle).round() as i32, ((1.0 - r) * gene_a_power + r * gene_b_power).round()  as i32);
        }
        return childs;
    }

    fn mutate(&mut self, mutation_rate: f32, rng: &mut ThreadRng) {
        for i in 0..self.chromosome.genes.len() {
            if rng.gen_bool(mutation_rate as f64) {
                self.chromosome.genes[i].0 = rng.gen_range(-15..16);

            }
            if rng.gen_bool(mutation_rate as f64)  {
                self.chromosome.genes[i].1 = rng.gen_range(-1..2);
            }
        }
    }
}

/* --------------------------------------------------------- */
/* -   GAME   ---------------------------------------------- */
/* --------------------------------------------------------- */

pub struct Game {
    pub level_data: LevelData,
    pub gravity: f32,
    pub map: Vec<Pos>,
    pub landing_zone_xmin: f32,
    pub landing_zone_xmax: f32,
    pub landing_zone_y: f32,
    pub landing_zone_index: usize,
    pub surface_length: i32,
    pub ships: Vec<Ship>,
    pub turn: usize,
    pub paused: bool,
    pub next_turn: bool,
    pub mutation_rate: f32,
    pub generation: i32,
    pub search_ended: bool,
    pub best_ship: Option<Ship>,
    pub previous_population: Vec<Ship>,
    pub rng: ThreadRng
}

impl Game {
    pub fn setup(level_data: &LevelData) -> Self {
        let mut landing_zone_xmin = 0.0;
        let mut landing_zone_xmax = 0.0;
        let mut landing_zone_y = 0.0;
        let mut landing_zone_index = 0;
        let mut surface_length = 0;
        for i in 0..(level_data.map.len() - 1) {
            if level_data.map[i].y == level_data.map[i + 1].y {
                landing_zone_xmin = level_data.map[i].x;
                landing_zone_xmax = level_data.map[i + 1].x;
                landing_zone_y = level_data.map[i].y;
                landing_zone_index = i;
            }
            let x_length = level_data.map[i].x - level_data.map[i + 1].x;
            let y_length = level_data.map[i].y - level_data.map[i + 1].y;
            surface_length += ((x_length * x_length) + (y_length * y_length)).sqrt() as i32;
        }
        let mut rng = rand::thread_rng();
        return Self {
            level_data: level_data.clone(),
            gravity: 3.711,
            map: level_data.map.clone(),
            landing_zone_xmin,
            landing_zone_xmax,
            landing_zone_y,
            landing_zone_index,
            surface_length,
            ships: (0..POPULATION_COUNT).map(|_| Ship::new(level_data, &mut rng)).collect::<Vec<Ship>>(),
            turn: 0,
            paused: true,
            next_turn: false,
            mutation_rate: MUTATION_RATE,
            generation: 0,
            search_ended: false,
            best_ship: None,
            previous_population: vec![],
            rng: rng
        }
    }
    
    fn pick_partner(&mut self) -> Ship {
        let mut rng = rand::thread_rng();
        let mut selected: Option<Ship> = None;
        let r: f32 = rng.gen();
        if r < self.ships[0].chromosome.prob {
            return self.ships[0].clone();
        }
        for i in 1..POPULATION_COUNT {
            if self.ships[i - 1].chromosome.prob < r && r <= self.ships[i].chromosome.prob {
                selected = Some(self.ships[i].clone());
                break;
            }
        }
        return selected.unwrap();
    }

    pub fn get_elites(&self) -> Vec<Ship> {
        let mut ships = self.ships.clone();
        let mut elites = vec![];
        ships.sort_by(|a, b| b.chromosome.fitness.partial_cmp(&a.chromosome.fitness).unwrap());
        for i in 0..(POPULATION_COUNT as f32 * ELITE_PERCENTAGE) as usize {
            elites.push(ships[i].clone());
        }
        return elites;
    }
    
    pub fn generate(&mut self) {
        let mut new_ships: Vec<Ship> = vec![];
        let mut fitness_sum: f32 = 0.0;
        for i in 0..POPULATION_COUNT {
            fitness_sum += self.ships[i].chromosome.fitness;
        }
        let mut prob_sum = 0.0;
        for i in 0..POPULATION_COUNT {
            self.ships[i].chromosome.prob = prob_sum + self.ships[i].chromosome.fitness / fitness_sum;
            prob_sum += self.ships[i].chromosome.prob;
        }
    
        for _ in (0..POPULATION_COUNT).step_by(2) {
            let partner_a = self.pick_partner();
            let mut partner_b = self.pick_partner();
            while partner_a == partner_b {
                partner_b = self.pick_partner();
            }
            let mut childs: [Ship; 2] = partner_a.crossover(partner_b, &self.level_data, &mut self.rng);
            childs[0].mutate(self.mutation_rate, &mut self.rng);
            childs[1].mutate(self.mutation_rate, &mut self.rng);
            new_ships.push(childs[0].clone());
            new_ships.push(childs[1].clone());
        }
        self.ships = new_ships;
        self.generation += 1;
    }

    fn calc_min_dist(&self, crash_pos: &Pos, crash_zone_index: usize) -> f32 {
        if crash_zone_index == self.landing_zone_index {
            return 0.0;
        }

        let dir: i32 = match crash_zone_index > self.landing_zone_index {
            true => -1, // crashed on right of landing zone
            _ => 1// crashed on right of landing zone
        };

        let offset = match dir { // position on the crash_zone
            -1 => { // going left
                let dist_x = crash_pos.x - self.map[crash_zone_index].x;
                let dist_y = crash_pos.y - self.map[crash_zone_index].y;
                ((dist_x * dist_x) + (dist_y * dist_y)).sqrt()
            },
            _ => { // going right
                let dist_x = crash_pos.x - self.map[crash_zone_index + 1].x;
                let dist_y = crash_pos.y - self.map[crash_zone_index + 1].y;
                ((dist_x * dist_x) + (dist_y * dist_y)).sqrt()
            }
        };

        let mut dist = 0.0;
        let mut prev: i32 = crash_zone_index as i32;
        let mut index: i32 = crash_zone_index as i32 + dir;
        while index >= 0 && index < self.map.len() as i32 && index != self.landing_zone_index as i32 {
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
        if self.ships[ship_index].is_out {
            self.ships[ship_index].chromosome.fitness = 1.0;
        } else if self.ships[ship_index].crash_zone_index != self.landing_zone_index {
            // eprintln!("A");
            let dist = self.calc_min_dist(&self.ships[ship_index].crash_pos, self.ships[ship_index].crash_zone_index);
            let dist_score = scale(dist, 0.0, self.surface_length as f32, 99.0, 0.0); // 0 to 99.0
            let speed = ((self.ships[ship_index].h_speed * self.ships[ship_index].h_speed) + (self.ships[ship_index].v_speed * self.ships[ship_index].v_speed)).sqrt(); // 0 to 707.106781187
            let mut speed_score = 0.0;
            if speed > 100.0 {
                speed_score = 0.1 * speed;
            }
            self.ships[ship_index].chromosome.fitness = 1.0 + dist_score - speed_score; // 1 to 100.0
        } else if !self.ships[ship_index].is_solution {
            // eprintln!("B");
            let mut x_score = 50.0;
            if (self.ships[ship_index].h_speed).abs() > 20.0 {
                x_score = scale(self.ships[ship_index].h_speed.abs(), 500.0, 20.0, 0.0, 50.0); // 0 to 50.0
            }
            let mut y_score = 50.0;
            if self.ships[ship_index].v_speed < -40.0 {
                y_score = scale(self.ships[ship_index].v_speed, -500.0, -40.0, 0.0, 50.0); // 0 to 50.0
            }
            // let angle_score = scale(self.ships[ship_index].angle, -90.0, 90.0, 0.0, 5.0); // 0 to 5.0
            self.ships[ship_index].chromosome.fitness = 100.0 + x_score + y_score/* - angle_score*/; // 100 to 200
        } else {
            let fuel_score = scale(self.ships[ship_index].fuel, 0.0, self.level_data.fuel, 0.0, 100.0); // 0 to 100.0
            self.ships[ship_index].chromosome.fitness = 200.0 + fuel_score; // 200 to 300
        }
    }
    
    pub fn evaluate(&mut self) {
        let mut max_fitness: f32 = 0.0;
        let mut total_fitness: f32 = 0.0;
    
        for i in 0..POPULATION_COUNT {
            self.calc_fitness(i);
            if self.ships[i].chromosome.fitness > max_fitness {
                max_fitness = self.ships[i].chromosome.fitness;
            }
            total_fitness += self.ships[i].chromosome.fitness;
        }
        let fitness_average: i32 = (total_fitness / POPULATION_COUNT as f32) as i32;
        eprintln!("gen: {} | av: {} | max: {}", self.generation, fitness_average, max_fitness as i32);
    }
}