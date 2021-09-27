use super::pos::*;

fn on_segment(p: &Pos, q: &Pos, r: &Pos) -> bool {
    return q.x <= p.x.max(r.x) && q.x >= p.x.min(r.x) && q.y <= p.y.max(r.y) && q.y >= p.y.min(r.y);
}

fn orientation(p0: &Pos, p1: &Pos, r: &Pos) -> i32 {
    return match (p1.y - p0.y) * (r.x - p1.x) - (p1.x - p0.x) * (r.y - p1.y) {
        v if v > 0.0 => 1,
        v if v < 0.0 => 2,
        _ => 0
    };
}
 
pub fn do_intersect(a0: &Pos, a1: &Pos, b0: &Pos, b1: &Pos) -> bool {
    let o1: i32 = orientation(a0, a1, b0);
    let o2: i32 = orientation(a0, a1, b1);
    let o3: i32 = orientation(b0, b1, a0);
    let o4: i32 = orientation(b0, b1, a1);
 
    return (o1 != o2 && o3 != o4)
        || (o1 == 0 && on_segment(a0, b0, a1))
        || (o2 == 0 && on_segment(a0, b1, a1))
        || (o3 == 0 && on_segment(b0, a0, b1))
        || (o4 == 0 && on_segment(b0, a1, b1)
    );
}

// segment has to intersect
pub fn find_intersection_point(a: &Pos, b: &Pos, c: &Pos, d: &Pos) -> Pos {
    // Line AB represented as a1x + b1y = c1
    let a1: f32 = b.y - a.y;
    let b1: f32 = a.x - b.x;
    let c1: f32 = a1 * a.x + b1 * a.y;
  
    // Line CD represented as a2x + b2y = c2
    let a2: f32 = d.y - c.y;
    let b2: f32 = c.x - d.x;
    let c2: f32 = a2 * c.x + b2 * c.y;
  
    let determinant: f32 = a1 * b2 - a2 * b1;
    let x: f32 = (b2 * c1 - b1 * c2) / determinant;
    let y: f32 = (a1 * c2 - a2 * c1) / determinant;

    return Pos::from(x, y);
}

pub fn scale(value: f32, from1: f32, to1: f32, from2: f32, to2: f32,) -> f32 {
    return (value - from1) / (to1 - from1) * (to2 - from2) + from2;
}