use rand::Rng;

// constants
pub const INF: f64 = f64::INFINITY;
pub const PI: f64 = 3.1415926535897932385;

// utility functions
pub fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * PI / 180.0
}

pub fn random_float() -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..100) as f64 / 100.0
}

pub fn random_range(min: f64, max: f64) -> f64 {
    min + (max-min) * random_float()
}
pub fn random_int(min: f64, max: f64) -> i32 {
    random_range(min, max+1.0) as i32
}

pub fn clamp(x: f64, min: f64, max: f64) -> f64 {
    if x > max {return max}
    if x < min {return min}
    x
}