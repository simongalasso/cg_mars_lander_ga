extern crate glutin_window;
extern crate opengl_graphics;
extern crate piston;

use piston::event_loop::*;
use piston::input::*;

mod display;
mod maths;

use crate::display::display::*;

fn main() {
    let mut display: Display = Display::setup(7000.0 * SCREEN_SCALE, 3000.0 * SCREEN_SCALE);

    let mut gravity: f64 = 3.711;
    let mut ship_pos: Pos = Pos::from(2500.0, 2700.0);
    let mut h_speed: f64 = 0.0; // in m/s
    let mut v_speed: f64 = 0.0; // in m/s
    let mut fuel: f64 = 550.0;
    let mut angle: f64 = 0.0;
    let mut power: f64 = 0.0;
    
    let mut map = vec![
        Pos::from(0.0, 100.0),
        Pos::from(1000.0, 500.0),
        Pos::from(1500.0, 1500.0),
        Pos::from(3000.0, 1000.0),
        Pos::from(4000.0, 150.0),
        Pos::from(5500.0, 150.0),
        Pos::from(6999.0, 800.0)
    ];

    eprintln!("x: {}, y: {}", ship_pos.x.round(), ship_pos.y.round());
    eprintln!("v_speed: {}", v_speed);

    // (angle, power)
    let instructions = vec![
        (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0),
        (-35, 3), (-35, 3), (-35, 3), (-35, 3), (-35, 3),
        (-35, 3), (-35, 3), (-35, 3), (-35, 3), (-35, 3),
        (-35, 3), (-35, 3), (-35, 3), (-35, 3), (-35, 3),
        (-35, 3), (-35, 3), (-35, 3), (-35, 3), (-35, 3),
        (-35, 3), (-35, 3), (-35, 3), (-35, 3), (-35, 3),
        (-35, 3), (-35, 3), (-35, 3), (-35, 3), (-35, 3)
    ];

    let mut turn: i32 = 0;
    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut display.window) {
        if let Some(event) = e.render_args() {
            display.gl.draw(event.viewport(), |_context, gl| {
                graphics::clear(GREY1, gl);
            });
            display.render_ground(&event, &map);
            display.render_ship(&event, &ship_pos, angle, power);
        }
        // if let Some(event) = e.update_args() {
        // }
        if let Some(args) = e.press_args() {
            match args {
                Button::Keyboard(Key::Right) => {
                    // let instruction = instructions[turn as usize];
                    // let rot: f64 = instruction.0 as f64;
                    // let thrust: f64 = instruction.1 as f64;
                    let rot: f64 = -5.0;
                    let thrust: f64 = 4.0;
                    
                    angle += match rot {
                        target_angle if target_angle > 0.0 => (target_angle as f64 - angle).min(15.0),
                        target_angle => (target_angle - angle).max(-15.0)
                    };
                    power += match thrust {
                        target_thrust if target_thrust > 0.0 => (target_thrust as f64 - power).min(1.0),
                        target_thrust => (target_thrust - power).max(-1.0)
                    };
                    fuel -= power;

                    let v_acc = (power * (angle.to_radians()).cos()) - gravity;
                    // eprintln!("v_acc: {}", v_acc);
                    ship_pos.y = ship_pos.y + v_speed + 0.5 * v_acc;
                    v_speed += v_acc;

                    let h_acc = power * (-angle.to_radians()).sin();
                    // eprintln!("h_acc: {}", h_acc);
                    ship_pos.x = ship_pos.x + h_speed + 0.5 * h_acc;
                    h_speed += h_acc;

                    eprintln!("turn: {}", turn);
                    eprintln!("x: {}, y: {}", ship_pos.x.round(), ship_pos.y.round());
                    eprintln!("v_speed: {}", v_speed.round());
                    eprintln!("h_speed: {}", h_speed.round());
                    eprintln!("angle: {}", angle);
                    eprintln!("power: {}", power);
                    eprintln!("fuel: {}", fuel);
                    eprintln!("---");
                    display.update(&ship_pos);
                    turn += 1;
                },
                // Button::Keyboard(Key::Left) => {
                // }
                _ => {}
            }
        }
    }
}