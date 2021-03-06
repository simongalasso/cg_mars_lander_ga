use super::utils::*;
use super::space::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
}

impl Pos {
    pub fn from(x: f32, y: f32) -> Self {
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