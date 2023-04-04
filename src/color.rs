use std::cmp::Ordering;
use crate::vec3::Vec3;
use crate::mat3::Mat3;

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    pub fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }

    pub fn black() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
    }

    pub fn gray() -> Self {
        Self::new(0.5, 0.5, 0.5, 1.0)
    }

    pub fn green() -> Self {
        Self::new(0.0, 1.0, 0.0, 1.0)
    }

    pub fn sum(&self) -> f64 {
        self.r + self.g + self.b
    }

    pub fn max(&self) -> f64 {
        if self.r > self.g && self.r > self.b{
            return self.r
        } else if self.g > self.r && self.g > self.b {
            return self.g
        } else {
            return self.b
        }
    }

    pub fn has_nan(&self) -> bool {
        if self.r.is_nan() || self.g.is_nan() || self.b.is_nan() || self.a.is_nan() {
            return true
        }
        false
    }

    pub fn to_normal_vec(&self, tangent: Vec3, bitangent: Vec3, surface_normal: Vec3) -> Vec3 {
        let x = self.r;
        let y = self.g;
        let z = self.b;

        // remap from [0, 1] to [-1, 1]
        let tangent_normal = Vec3::new(x * 2.0 - 1.0, y * 2.0 - 1.0, z * 2.0 - 1.0);

        // construct the transformation matrix
        let tangent_matrix = Mat3::new([
            [tangent.x, bitangent.x, surface_normal.x],
            [tangent.y, bitangent.y, surface_normal.y],
            [tangent.z, bitangent.z, surface_normal.z]
            ]);

        // transform the tangent space normal to world space
        let world_normal = tangent_matrix * tangent_normal;

        return world_normal
    }
}

impl std::ops::Add for Color {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
            a: self.a + other.a,
        }
    }
}

impl std::ops::Add<f64> for Color {
    type Output = Self;

    fn add(self, other: f64) -> Self {
        Self {
            r: self.r + other,
            g: self.g + other,
            b: self.b + other,
            a: self.a + other,
        }
    }
}

impl std::ops::Sub for Color {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            r: self.r - other.r,
            g: self.g - other.g,
            b: self.b - other.b,
            a: self.a - other.a,
        }
    }
}

impl std::ops::Mul<f64> for Color {
    type Output = Self;

    fn mul(self, t: f64) -> Self {
        Self {
            r: self.r * t,
            g: self.g * t,
            b: self.b * t,
            a: self.a
        }
    }
}

impl std::ops::Mul<Color> for Color {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            r: self.r * other.r,
            g: self.g * other.g,
            b: self.b * other.b,
            a: self.a * other.a,
        }
    }
}

impl std::ops::Div<f64> for Color {
    type Output = Color;

    fn div(self, rhs: f64) -> Color {
        Color {
            r: self.r / rhs,
            g: self.g / rhs,
            b: self.b / rhs,
            a: self.a
        }
    }
}

impl std::ops::Div<u32> for Color {
    type Output = Color;

    fn div(self, rhs: u32) -> Color {
        Color {
            r: self.r / rhs as f64,
            g: self.g / rhs as f64,
            b: self.b / rhs as f64,
            a: self.a / rhs as f64,
        }
    }
}

impl std::ops::Div<Color> for Color {
    type Output = Self;

    fn div(self, other: Self) -> Color {
        Color {
            r: self.r / other.r,
            g: self.g / other.g,
            b: self.b / other.b,
            a: self.a / other.a
        }
    }
}