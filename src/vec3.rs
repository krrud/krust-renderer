use crate::utility::{random_float, random_range};
use std::f64;
use std::{ops, cmp};

#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }

    pub fn black() -> Vec3 {
        Vec3::new(0.0, 0.0, 0.0)
    }

    pub fn white() -> Vec3 {
        Vec3::new(1.0, 1.0, 1.0)
    }

    pub fn gray() -> Vec3 {
        Vec3::new(0.5, 0.5, 0.5)
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn z(&self) -> f64 {
        self.z
    }

    pub fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn dot(&self, other: &Vec3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn unit_vector(&self) -> Vec3 {
        let length = self.length();
        Vec3::new(self.x / length, self.y / length, self.z / length)
    }

    pub fn random_in_unit_sphere() -> Vec3 {
        loop {
            let p: Vec3 = Self::rand_range(-1.0, 1.0);
            if p.length_squared() < 1.0 {
                return p;
            }
        }
    }

    pub fn random_unit_vector2() -> Vec3 {
        let rand = Vec3::random_in_unit_sphere();
        let mut scatter = Vec3::unit_vector(&rand);
        if Vec3::near_zero(&scatter)
            || scatter.x().is_nan()
            || scatter.y().is_nan()
            || scatter.z().is_nan()
        {
            scatter = Vec3::random_unit_vector();
        }
        scatter
    }

    pub fn random_unit_vector() -> Vec3 {
        let rand = Vec3::random_in_unit_sphere();
        let mut scatter = Vec3::unit_vector(&rand);
        if Vec3::near_zero(&scatter)
            || scatter.x().is_nan()
            || scatter.y().is_nan()
            || scatter.z().is_nan()
        {
            let rand = Vec3::random_in_unit_sphere();
            scatter = Vec3::unit_vector(&rand);
        }
        scatter
    }

    pub fn rand() -> Vec3 {
        Vec3 {
            x: random_float(),
            y: random_float(),
            z: random_float(),
        }
    }

    pub fn rand_range(min: f64, max: f64) -> Vec3 {
        Vec3 {
            x: random_range(min, max),
            y: random_range(min, max),
            z: random_range(min, max),
        }
    }

    pub fn near_zero(&self) -> bool {
        let s = 1e-8;
        self.x < s && self.y < s && self.z < s
    }

    pub fn reflect(v: Vec3, n: Vec3) -> Vec3 {
        v - (n * Vec3::dot(&v, &n)) * 2.0
    }

    pub fn reflect2(incident: Vec3, normal: Vec3) -> Vec3 {
        incident - 2.0 * incident.dot(&normal) * normal
    }

    pub fn refract(uv: &Vec3, n: &Vec3, etai_over_etat: f64) -> Vec3 {
        let neg_uv = *uv*-1.0;
        let cos_theta = f64::min(Vec3::dot(&neg_uv, n), 1.0);
        let r_out_perp =  (*uv + *n * cos_theta) * etai_over_etat;
        let r_out_parallel = *n * ((1.0 - r_out_perp.length_squared())).abs().sqrt() * -1.0;
        r_out_perp + r_out_parallel
    }

    pub fn random_in_unit_disk() -> Vec3 {
        loop {
            let p = Vec3::new(random_range(-1.0,1.0), random_range(-1.0,1.0), 0.0);
            if p.length_squared() >= 1.0 {continue;}
            return p;
        }
    }
}

impl ops::Add for Vec3 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl ops::Add<Vec<f64>> for Vec3 {
    type Output = Vec<f64>;
    fn add(self, other: Vec<f64>) -> Vec<f64> {
        vec![
            other[0] + self.x,
            other[1] + self.y,
            other[2] + self.z,
            other[3]
        ] 
    }
}

impl ops::Sub for Vec3 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl ops::Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Vec3 {
        Vec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl ops::Mul<Vec3> for Vec3 {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }
}

impl ops::Mul<Vec<f64>> for Vec3 {
    type Output = Vec<f64>;
    fn mul(self, other: Vec<f64>) -> Vec<f64> {
        vec![
            other[0] * self.x,
            other[1] * self.y,
            other[2] * self.z,
            other[3]
        ]
    }
}

impl ops::Mul<f64> for Vec3 {
    type Output = Vec3;
    fn mul(self, other: f64) -> Vec3 {
        Vec3 {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}

impl ops::Mul<Vec3> for f64 {
    type Output = Vec3;
    fn mul(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: other.x * self,
            y: other.y * self,
            z: other.z * self,
        }
    }
}

impl ops::Div<Vec3> for Vec3 {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        Self {
            x: self.x / other.x,
            y: self.y / other.y,
            z: self.z / other.z,
        }
    }
}

impl ops::Div<f64> for Vec3 {
    type Output = Self;
    fn div(self, other: f64) -> Self {
        Self {
            x: self.x / other,
            y: self.y / other,
            z: self.z / other,
        }
    }
}
