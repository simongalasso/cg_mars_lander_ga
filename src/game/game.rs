extern crate rand;

use crate::maths::pos::*;
use rand::Rng;

/* --------------------------------------------------------- */
/* -   CHROMOSOME   ---------------------------------------- */
/* --------------------------------------------------------- */

#[derive(Debug, Clone, PartialEq)]
pub struct Chromosome {
    pub genes: Vec<(i32, i32)>, // angle, thrust
    pub fitness: f64,
    pub prob: f64
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
    pub chromosome: Chromosome,
    pub pos: Pos,
    pub angle: f64,
    pub power: f64,
    pub h_speed: f64,
    pub v_speed: f64,
    pub fuel: f64,
    pub is_dead: bool,
    pub is_solution: bool,
    pub crash_pos: Pos,
    pub path: Vec<Pos>,
    pub is_best: bool,
    pub is_elite: bool,
    pub crash_zone_index: usize
}

impl Ship {
    fn new() -> Self {
        return Self {
            chromosome: Chromosome::new(),
            pos: Pos::from(2500.0, 2700.0),
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
            is_elite: false,
            crash_zone_index: 0
        }
    }

    pub fn is_out_of_map(&self) -> bool {
        return self.pos.x < 0.0 || self.pos.x >= 7000.0 || self.pos.y < 0.0 || self.pos.y >= 3000.0;
    }

    pub fn simulate(&mut self, angle: f64, power: f64, gravity: f64) {
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

pub const POPULATION_COUNT: usize = 100;
pub const CHROMOSOME_SIZE: usize = 500;
pub const RAYS_MODE: i32 = 0;
pub const SHIPS_MODE: i32 = 1;

pub struct Game {
    pub display_mode: i32,
    pub gravity: f64,
    pub map: Vec<Pos>,
    pub landing_zone_xmin: f64,
    pub landing_zone_xmax: f64,
    pub landing_zone_y: f64,
    pub ships: Vec<Ship>,
    pub turn: i32,
    pub paused: bool,
    pub next_turn: bool,
    pub mutation_rate: f64,
    pub generation: i32,
    pub search_ended: bool,
    pub best_ship: Option<Ship>
}

impl Game {
    pub fn setup() -> Self {
        return Self {
            display_mode: RAYS_MODE,
            gravity: 3.711,
            map: vec![Pos {
                x: 0.0,
                y: 100.0,
            },
            Pos {
                x: 1000.0,
                y: 500.0,
            },
            Pos {
                x: 1500.0,
                y: 1500.0,
            },
            Pos {
                x: 3000.0,
                y: 1000.0,
            },
            Pos {
                x: 4000.0,
                y: 150.0,
            },
            Pos {
                x: 5500.0,
                y: 150.0,
            },
            Pos {
                x: 6999.0,
                y: 800.0,
            }],
            landing_zone_xmin: 4000.0,
            landing_zone_xmax: 5500.0,
            landing_zone_y: 150.0,
            ships: (0..POPULATION_COUNT).map(|_| Ship::new()).collect::<Vec<Ship>>(),
            turn: 0,
            paused: true,
            next_turn: false,
            mutation_rate: 0.01,
            generation: 0,
            search_ended: false,
            best_ship: None
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

    pub fn get_elites(&self) -> Vec<Ship> {
        let mut ships = self.ships.clone();
        let mut elites = vec![];
        ships.sort_by(|a, b| b.chromosome.fitness.partial_cmp(&a.chromosome.fitness).unwrap());
        for i in 0..(POPULATION_COUNT as f64 * 0.2) as usize {
            elites.push(ships[i].clone());
        }
        return elites;
    }
    
    pub fn generate(&mut self) {
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

        let dir: i32 = match crash_zone_index > landing_zone_index {
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
    
    pub fn evaluate(&mut self) -> usize {
        let mut max_fitness: f64 = 0.0;
        // let mut total_fitness: f64 = 0.0;
        let mut best = 0;
    
        for i in 0..POPULATION_COUNT {
            self.calc_fitness(i);
            if self.ships[i].chromosome.fitness > max_fitness {
                max_fitness = self.ships[i].chromosome.fitness;
                best = i;
            }
            // total_fitness += self.ships[i].chromosome.fitness;
        }
        // let fitness_average: i32 = (total_fitness / POPULATION_COUNT as f64) as i32;
        // println!("gen: {} | av: {} | max: {}", self.generation, fitness_average, max_fitness as i32);
        return best;
    }
}