use std::fs::File;
use std::fs::metadata;
use std::io::prelude::*;
use std::io::BufReader;

use crate::maths::pos::*;

macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

#[derive(Clone)]
pub struct LevelData {
    pub pos: Pos,
    pub angle: f32,
    pub power: f32,
    pub h_speed: f32,
    pub v_speed: f32,
    pub fuel: f32,
    pub map: Vec<Pos>
}

pub fn parse_file(dataset_file: &str) -> Result<LevelData, String> {
    if !metadata(dataset_file).expect("error: a problem occured with the file").is_file() {
        return Err(String::from("error: the file should be a file"));
	}
    let file = File::open(dataset_file).expect("error: file not found");
	let mut lines = BufReader::new(file);

    let mut input_line = String::new();
    lines.read_line(&mut input_line).unwrap();
    let inputs = input_line.trim_end().split(" ").collect::<Vec<_>>();
    let x: f32 = parse_input!(inputs[0], f32);
    let y: f32 = parse_input!(inputs[1], f32);

    let mut input_line = String::new();
    lines.read_line(&mut input_line).unwrap();
    let inputs = input_line.trim_end().split(" ").collect::<Vec<_>>();
    let angle: f32 = parse_input!(inputs[0], f32);
    let power: f32 = parse_input!(inputs[1], f32);
    let h_speed: f32 = parse_input!(inputs[2], f32);
    let v_speed: f32 = parse_input!(inputs[3], f32);
    let fuel: f32 = parse_input!(inputs[4], f32);

    let mut input_line = String::new();
    lines.read_line(&mut input_line).unwrap();
    let inputs = input_line.trim_end().split(",").collect::<Vec<_>>();
    let map = inputs.iter().map(|input| {
        let values = input.split(" ").collect::<Vec<_>>();
        Pos::from(parse_input!(values[0], f32), parse_input!(values[1], f32))
    }).collect::<Vec<Pos>>();

    return Ok(LevelData {
        pos: Pos::from(x, y),
        angle, power, h_speed, v_speed, fuel,
        map
    });
}