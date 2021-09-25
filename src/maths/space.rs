#[derive(Debug, Clone, PartialEq)]
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