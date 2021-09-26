#[derive(Debug, Clone, PartialEq)]
pub struct Space {
    pub x0: f32,
    pub x1: f32,
    pub y0: f32,
    pub y1: f32,
}

impl Space {
    pub fn new(x0: f32, x1: f32, y0: f32, y1: f32) -> Self {
        return Space {
            x0: x0,
            x1: x1,
            y0: y0,
            y1: y1
        }
    }
}