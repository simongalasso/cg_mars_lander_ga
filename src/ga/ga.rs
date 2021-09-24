extern crate rand;

use rand::Rng;
use super::super::{Ship, Game};

/* Entity ------------------------------------------------ */

#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    pub genes: Vec<(i32, i32)>, // angle, thrust
    pub fitness: f64,
}

impl Entity {
    fn new(length: usize) -> Self {
        let mut genes = vec![];
        let mut prev_angle: i32 = 0;
        let mut prev_power: i32 = 0;
        for _ in 0..length {
            let rand_angle = rand::thread_rng().gen_range(-15..=15);
            let rand_power = rand::thread_rng().gen_range(-1..=1);
            let mut new_angle = prev_angle + rand_angle;
            if new_angle > 90 {
                new_angle = 90;
            } else if new_angle < -90 {
                new_angle = -90;
            }
            let mut new_power = prev_power + rand_power;
            if new_power > 4 {
                new_power = 4;
            } else if new_power < 0 {
                new_power = 0;
            }
            genes.push((new_angle, new_power));
            prev_angle = new_angle;
            prev_power = new_power;
        }
        return Self {
            genes: genes,
            fitness: 0.0
        };
    }

    pub fn calc_fitness(&mut self, ship: &Ship, lz_xmin: f64, lz_xmax: f64) {
        if ship.crash_pos.x < 0.0 || ship.crash_pos.x >= 7000.0 || ship.crash_pos.y < 0.0 || ship.crash_pos.y >= 7000.0 {
            self.fitness = 1.0;
            return;
        }

        let speed = ((ship.h_speed * ship.h_speed) + (ship.v_speed * ship.v_speed)).sqrt();

        if ship.crash_pos.x < lz_xmin || ship.crash_pos.x > lz_xmax {
            let norm_dist_x = (ship.crash_pos.x - (lz_xmax + lz_xmin) / 2.0).abs() / 7000.0; // 0.0 to 1.0
            // let dist_y = (ship.crash_pos.y - game.landing_zone_y).abs(); // 3000.0 to 0.0
            let mut score = 100.0 - (norm_dist_x * 100.0);
            let speed_score: f64 = 0.1 * (speed - 100.0).max(0.0);
		    score -= speed_score;
            self.fitness = score;
            return;
        } else if !ship.is_solution {
            let mut x_score = 0.0;
            if 20.0 < (ship.h_speed).abs() {
                x_score = ((ship.h_speed).abs() - 20.0) / 2.0;
            }
            let mut y_score = 0.0;
            if ship.v_speed < -40.0 {
                y_score = (-40.0 - ship.v_speed) / 2.0;
            }
            self.fitness = 200.0 - x_score - y_score;
        } else {
            self.fitness = 200.0 + (100.0 * ship.fuel / 550.0);
        }
    }

    fn crossover(&self, partner: Entity) -> [Entity; 2] {
        let mut childs: [Entity; 2] = [Entity::new(self.genes.len()), Entity::new(self.genes.len())];

        for i in 0..self.genes.len() {
            let parent_a_angle = self.genes[i].0 as f64;
            let parent_a_power = self.genes[i].1 as f64;
            let parent_b_angle = partner.genes[i].0 as f64;
            let parent_b_power = partner.genes[i].1 as f64;
            let r: f64 = rand::thread_rng().gen_range(0.0..=1.0);

            childs[0].genes[i as usize] = ((r * parent_a_angle + (1.0 - r) * parent_b_angle).round() as i32, (r * parent_a_power + (1.0 - r) * parent_b_power).round()  as i32);
            childs[1].genes[i as usize] = (((1.0 - r) * parent_a_angle + r * parent_b_angle).round()  as i32, ((1.0 - r) * parent_a_power + r * parent_b_power).round()  as i32);
        }
        return childs;
    }

    fn mutate(&mut self, mutation_rate: f64) {
        let mut prev_angle: i32 = 0;
        let mut prev_power: i32 = 0;
        for i in 0..self.genes.len() {
            if rand::thread_rng().gen_range(0.0..=1.0) < mutation_rate {
                let rand_angle = rand::thread_rng().gen_range(-15..=15);
                let rand_power = rand::thread_rng().gen_range(-1..=1);
                let mut new_angle = prev_angle + rand_angle;
                if new_angle > 90 {
                    new_angle = 90;
                } else if new_angle < -90 {
                    new_angle = -90;
                }
                let mut new_power = prev_power + rand_power;
                if new_power > 4 {
                    new_power = 4;
                } else if new_power < 0 {
                    new_power = 0;
                }
                self.genes[i] = (new_angle, new_power);
            }
            prev_angle = self.genes[i].0;
            prev_power = self.genes[i].1;
        }
    }
}

/* Population -------------------------------------------- */

pub struct Population {
    pub mutation_rate: f64,
    pub entities: Vec<Entity>,
    pub generation: i32
}

impl Population {
    pub fn new(size: i32) -> Self {
        return Self {
            mutation_rate: 0.01,
            entities: (0..size).map(|_x| Entity::new(100)).collect(),
            generation: 0
        };
    }

    fn pick_partner(&mut self) -> Entity {
        // let mut entities = self.entities.clone();

        // let mut parent_a: Option<Entity> = None;
        // let mut parent_b: Option<Entity> = None;

        // let mut fitness_sum: f64 = 0.0;
        // for i in 0..entities.len() {
        //     fitness_sum += entities[i].fitness;
        // }

        // for i in 0..entities.len() {
        //     entities[i].fitness /= fitness_sum; 
        // }

        // entities.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());

        // for i in 0..entities.len() {
        //     let mut sum = 0.0;
        //     for j in i..entities.len() {
        //         sum += entities[j].fitness;
        //     }
        //     entities[i].fitness = sum;
        // }

        // for n in 0..entities.len() {
        //     eprint!("{:.3}, ", entities[n].fitness);
        // }
        // eprintln!();

        // let r: f64 = rand::thread_rng().gen_range(0.0..=1.0);
        // eprintln!("r: {}", r);
        // for (i, e) in entities.iter().enumerate() {
        //     if e.fitness > r {
        //         parent_a = Some(e.clone());
        //         eprint!("{}, ", i);
        //         break;
        //     }
        // }

        // 'outer: while parent_b.is_none() {
        //     let r: f64 = rand::thread_rng().gen_range(0.0..=1.0);
        //     for (i, e) in entities.iter().enumerate() {
        //         if e.fitness > r {
        //             if e != parent_a.as_ref().unwrap() {
        //                 parent_b = Some(e.clone());
        //                 eprintln!("{}", i);
        //                 break 'outer;
        //             }
        //         }
        //     }
        // }
        // return (parent_a.unwrap(), parent_b.unwrap());
        let mut entities = self.entities.clone();

        let mut fitness_sum: f64 = 0.0;
        for i in 0..entities.len() {
            fitness_sum += entities[i].fitness;
        }

        for i in 0..entities.len() {
            entities[i].fitness /= fitness_sum; 
        }

        let mut r: f64 = rand::thread_rng().gen_range(0.0..=1.0);
        let mut index = 0;
        while r > 0.0 {
            r -= entities[index].fitness;
            index += 1;
        }
        return entities[index - 1].clone();
    }

    pub fn generate(&mut self) {
        let mut new_entities: Vec<Entity> = Vec::new();
        for _ in 0..(self.entities.len() as i32 / 2) {
            let partner_a = self.pick_partner();
            let mut partner_b = self.pick_partner();
            while partner_b == partner_a {
                partner_b = self.pick_partner();
            }
            let mut childs: [Entity; 2] = partner_a.crossover(partner_b);
            childs[0].mutate(self.mutation_rate);
            childs[1].mutate(self.mutation_rate);
            new_entities.push(childs[0].clone());
            new_entities.push(childs[1].clone());
        }
        self.entities = new_entities;
        self.generation += 1;
    }

    pub fn evaluate(&self) {
        let mut max_fitness: f64 = 0.0;
        let mut total_fitness: f64 = 0.0;

        for i in 0..self.entities.len() {
            if self.entities[i].fitness > max_fitness {
                max_fitness = self.entities[i].fitness;
            }
            total_fitness += self.entities[i].fitness;
        }
        let fitness_average: i32 = (total_fitness / self.entities.len() as f64) as i32;
        println!("gen: {} | av: {} | max: {}", self.generation, fitness_average, max_fitness as i32);
    }
}