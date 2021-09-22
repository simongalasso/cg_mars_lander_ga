pub fn scale(value: f64, from1: f64, to1: f64, from2: f64, to2: f64,) -> f64 {
    return (value - from1) / (to1 - from1) * (to2 - from2) + from2;
}