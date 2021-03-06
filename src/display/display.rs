extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use piston::window::WindowSettings;
use piston::input::*;
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use crate::display::display::graphics::Transformed;

use crate::game::game::*;
use crate::maths::pos::*;
use crate::maths::space::*;

pub const GREY1: [f32; 4] = [0.11, 0.11, 0.11, 1.0];
pub const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
pub const RED: [f32; 4] = [0.870, 0.152, 0.152, 1.0];
pub const GREEN: [f32; 4] = [0.345, 0.901, 0.196, 1.0];
pub const BLUE: [f32; 4] = [0.333, 0.623, 1.0, 1.0];
pub const GOLD: [f32; 4] = [1.0, 0.843, 0.0, 1.0];

pub const SCREEN_SCALE: f32 = 0.20;

pub struct Display {
    pub window_space: Space,
    pub window: GlutinWindow,
    pub gl: GlGraphics,
}

impl Display {
    pub fn setup(window_w: f32, window_h: f32) -> Self {
        let opengl: OpenGL = OpenGL::V3_2;
        let window: GlutinWindow = WindowSettings::new("Mars Lander Simulator", [window_w as f64, window_h as f64])
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

    pub fn clear_window(&mut self, event: &RenderArgs) {
        self.gl.draw(event.viewport(), |_context, gl| {
            graphics::clear(GREY1, gl);
        });
    }

    pub fn render_ground(&mut self, event: &RenderArgs, map: &Vec<Pos>) {
        let window_space = &self.window_space;
        self.gl.draw(event.viewport(), |c, gl| {
            for index in 0..(map.len() - 1) {
                let pos0: Pos = map[index].scale(window_space);
                let pos1: Pos = map[index + 1].scale(window_space);
                graphics::line(RED, 0.7, [
                    pos0.x as f64, pos0.y as f64,
                    pos1.x as f64, pos1.y as f64],
                c.transform, gl);
            }
        });
    }
    pub fn render_ship(&mut self, event: &RenderArgs, ship_pos: &Pos, ship_angle: f32, power: f32) {
        let window_space = &self.window_space;
        let rotation = -ship_angle;
        let (x, y) = (ship_pos.scale(window_space).x, ship_pos.scale(window_space).y);
        self.gl.draw(event.viewport(), |c, gl| {
            let transform = c
                .transform
                .trans(x as f64, y as f64)
                .rot_deg(rotation as f64)
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
            let rect = [5.0, 5.0, 8.0, power as f64 * 8.0];
            graphics::ellipse(WHITE, rect, transform, gl);
        });
    }

    pub fn render_ray(&mut self, event: &RenderArgs, ship: &Ship, color: [f32; 4]) {
        let window_space = &self.window_space;
        self.gl.draw(event.viewport(), |c, gl| {
            if ship.path.len() > 0 {
                for i in 0..(ship.path.len() - 1) {
                    let (x0, y0) = (ship.path[i].scale(window_space).x, ship.path[i].scale(window_space).y);
                    let (x1, y1) = (ship.path[i + 1].scale(window_space).x, ship.path[i + 1].scale(window_space).y);
                    graphics::line(color, 0.7, [x0 as f64, y0 as f64, x1 as f64, y1 as f64], c.transform, gl);
                }
            }
        });
    }
}