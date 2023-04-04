use std::ops::Mul;
use crate::vec3::Vec3;

#[derive(Copy, Clone, Debug)]
pub struct Mat3 {
    pub m: [[f64; 3]; 3],
}

impl Mul<Vec3> for Mat3 {
    type Output = Vec3;

    fn mul(self, v: Vec3) -> Self::Output {
        let x = self.m[0][0] * v.x + self.m[0][1] * v.y + self.m[0][2] * v.z;
        let y = self.m[1][0] * v.x + self.m[1][1] * v.y + self.m[1][2] * v.z;
        let z = self.m[2][0] * v.x + self.m[2][1] * v.y + self.m[2][2] * v.z;
        Vec3::new(x, y, z)
    }
}

impl Mat3 {
    pub fn new(m: [[f64; 3]; 3]) -> Self {
        Mat3 { m }
    }

    pub fn transpose(&self) -> Self {
        let mut res = Mat3::new([[0.0; 3]; 3]);
        for i in 0..3 {
            for j in 0..3 {
                res.m[i][j] = self.m[j][i];
            }
        }
        res
    }
}