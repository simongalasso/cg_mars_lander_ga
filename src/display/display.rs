extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use piston::window::WindowSettings;
use piston::input::*;
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use crate::display::display::graphics::Transformed;
use graphics::*;

use crate::maths::scale::*;

pub const GREY1: [f32; 4] = [0.11, 0.11, 0.11, 1.0];
pub const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
pub const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
pub const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

pub const SCREEN_SCALE: f64 = 0.20;

pub struct Space {
    pub x0: f64,
    pub x1: f64,
    pub y0: f64,
    pub y1: f64,
}

impl Space {
    pub fn new(x0: f64, x1: f64, y0: f64, y1: f64) -> Self {
        return Space {
            x0: x0,
            x1: x1,
            y0: y0,
            y1: y1
        }
    }
}

#[derive(Debug)]
pub struct Pos {
    pub x: f64,
    pub y: f64,
}

impl Pos {
    pub fn from(x: f64, y: f64) -> Self {
        return Pos {
            x: x,
            y: y
        }
    }

    pub fn scale(&self, space: &Space) -> Self {
        return Pos {
            x: scale(self.x, 0.0, 7000.0, space.x0, space.x1),
            y: scale(self.y, 0.0, 3000.0, space.y0, space.y1)
        }
    }
}

pub struct Display {
    pub window_space: Space,
    pub window: GlutinWindow,
    pub gl: GlGraphics,
}

impl Display {
    pub fn setup(window_w: f64, window_h: f64) -> Self {
        let opengl: OpenGL = OpenGL::V3_2;
        let window: GlutinWindow = WindowSettings::new("Mars Lander Simulator", [window_w, window_h])
            .graphics_api(opengl)
            .exit_on_esc(true)
            .build()
            .expect("error: can't initialize the GlutinWindow");
        return Display {
            window_space: Space::new(0.0, window_w, window_h, 0.0),
            window: window,
            gl: GlGraphics::new(opengl),
        };
    }

    pub fn render_ground(&mut self, event: &RenderArgs, map: &Vec<Pos>) {
        let window_space = &self.window_space;
        self.gl.draw(event.viewport(), |c, gl| {
            for index in 0..(map.len() - 1) {
                let pos0: Pos = map[index].scale(window_space);
                let pos1: Pos = map[index + 1].scale(window_space);
                graphics::line(RED, 0.7, [
                    pos0.x, pos0.y,
                    pos1.x, pos1.y],
                c.transform, gl);
            }
        });
    }
    pub fn render_ship(&mut self, event: &RenderArgs, ship_pos: &Pos, ship_angle: f64) {
        let window_space = &self.window_space;
        let square = rectangle::square(0.0, 0.0, 50.0);
        let rotation = -ship_angle;
        let (x, y) = (ship_pos.scale(window_space).x, ship_pos.scale(window_space).y);
        self.gl.draw(event.viewport(), |c, gl| {
            let transform = c
                .transform
                .trans(x, y)
                .rot_deg(rotation)
                .trans(-10.0, 0.0);
            graphics::line(WHITE, 0.7, [
                0.0, 0.0,
                20.0, 0.0],
                transform, gl);
            graphics::line(WHITE, 0.7, [
                0.0, 0.0,
                10.0, -30.0],
                transform, gl);
            graphics::line(WHITE, 0.7, [
                20.0, 0.0,
                10.0, -30.0],
                transform, gl);
        });
    }

    pub fn update(&mut self, ship_pos: &Pos) {
        // do stuff
    }
}