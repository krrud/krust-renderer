use crate::vec3::Vec3;

pub struct Onb {
    axis: [Vec3; 3],
}

impl Onb {
    pub fn new() -> Self {
        Self {
            axis: [Vec3::black(), Vec3::black(), Vec3::black()],
        }
    }

    pub fn u(&self) -> Vec3 {
        self.axis[0]
    }

    pub fn v(&self) -> Vec3 {
        self.axis[1]
    }

    pub fn w(&self) -> Vec3 {
        self.axis[2]
    }

    // pub fn local(&self, a: f64, b: f64, c: f64) -> Vec3 {
    //     a * self.u() + b * self.v() + c * self.w()
    // }

    pub fn local(&self, a: Vec3) -> Vec3 {
        a.x() * self.u() + a.y() * self.v() + a.z() * self.w()
    }

    pub fn build_from_w(&mut self, n: Vec3) {
        self.axis[2] = n.normalize();
        let a = if self.w().x().abs() > 0.9 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        self.axis[1] = Vec3::cross(&self.w(), &a).normalize();
        self.axis[0] = Vec3::cross(&self.w(), &self.v());
    }
}