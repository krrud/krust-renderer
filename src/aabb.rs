use crate::hit::HitRecord;
use crate::hit::Hittable;
use crate::material::{Material, Light};
use crate::ray::Ray;
use crate::vec3::Vec3;
use std::f64;
use std::mem;


#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub minimum: Vec3,
    pub maximum: Vec3,
}

impl Aabb {
    pub fn new(minimum: Vec3, maximum: Vec3) -> Aabb {
        Aabb { minimum, maximum }
    }

    pub fn empty() -> Aabb {
        Aabb {
            minimum: Vec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
            maximum: Vec3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
        }
    }

    pub fn bounding_box(&self, time0: f64, time1: f64) -> Aabb {
        Self::surrounding_box(*self, *self)
    }

    pub fn surrounding_box(box0: Aabb, box1: Aabb) -> Aabb {
        let small = Vec3::new(
            f64::min(box0.minimum.x, box1.minimum.x),
            f64::min(box0.minimum.y, box1.minimum.y),
            f64::min(box0.minimum.z, box1.minimum.z),
        );

        let big = Vec3::new(
            f64::max(box0.maximum.x, box1.maximum.x),
            f64::max(box0.maximum.y, box1.maximum.y),
            f64::max(box0.maximum.z, box1.maximum.z),
        );

        Aabb::new(small, big)
    }

    pub fn hit(&self, r: &Ray, mut t_min: f64, mut t_max: f64) -> (bool, Option<HitRecord>) {
        for a in 0..3 {
            let origin;
            let direction;
            let min;
            let max;
            if a == 0 {
                origin = r.origin.x;
                direction = r.direction.x;
                min = self.minimum.x;
                max = self.maximum.x;
            } else if a == 1 {
                origin = r.origin.y;
                direction = r.direction.y;
                min = self.minimum.y;
                max = self.maximum.y;
            } else {
                origin = r.origin.z;
                direction = r.direction.z;
                min = self.minimum.z;
                max = self.maximum.z;
            }
            let inv_d = 1.0 / direction;
            let mut t0 = (min - origin) * inv_d;
            let mut t1 = (max - origin) * inv_d;
            if inv_d < 0.0 {
                mem::swap(&mut t0, &mut t1);
            }
            t_min = if t0 > t_min { t0 } else { t_min };
            t_max = if t1 < t_max { t1 } else { t_max };
            if t_max <= t_min {
                return (false, None);
            }
        }
        (true, None)
    }
}
