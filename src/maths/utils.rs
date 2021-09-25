use super::pos::*;

fn on_segment(p: &Pos, q: &Pos, r: &Pos) -> bool {
    if q.x <= p.x.max(r.x) && q.x >= p.x.min(r.x) && q.y <= p.y.max(r.y) && q.y >= p.y.min(r.y) {
        return true;
    }
    return false;
}

fn orientation(p: &Pos, q: &Pos, r: &Pos) -> i32 {
    let val: f64 = (q.y - p.y) * (r.x - q.x) - (q.x - p.x) * (r.y - q.y);
    if val == 0.0 {
        return 0;
    }
    return if val > 0.0 { 1 } else { 2 };
}
 
pub fn do_intersect(p1: &Pos, q1: &Pos, p2: &Pos, q2: &Pos) -> bool {
    let o1: i32 = orientation(p1, q1, p2);
    let o2: i32 = orientation(p1, q1, q2);
    let o3: i32 = orientation(p2, q2, p1);
    let o4: i32 = orientation(p2, q2, q1);
 
    if (o1 != o2 && o3 != o4)
        || (o1 == 0 && on_segment(p1, p2, q1))
        || (o2 == 0 && on_segment(p1, q2, q1))
        || (o3 == 0 && on_segment(p2, p1, q2))
        || (o4 == 0 && on_segment(p2, q1, q2))
    {
        return true;
    }
    return false;
}

pub fn scale(value: f64, from1: f64, to1: f64, from2: f64, to2: f64,) -> f64 {
    return (value - from1) / (to1 - from1) * (to2 - from2) + from2;
}