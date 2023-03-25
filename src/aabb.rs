use crate::hit::HitRecord;
use crate::hit::Hittable;
use crate::material::{Material, Light};
use crate::ray::Ray;
use crate::vec3::Vec3;
use std::f64;
use std::mem;


#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn new(min: Vec3, max: Vec3) -> Aabb {
        Aabb { min, max }
    }

    pub fn empty() -> Aabb {
        Aabb {
            min: Vec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
            max: Vec3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
        }
    }

    pub fn bounding_box(&self, time0: f64, time1: f64) -> Aabb {
        Self::surrounding_box(*self, *self)
    }

    pub fn surrounding_box(box0: Aabb, box1: Aabb) -> Aabb {
        let small = Vec3::new(
            f64::min(box0.min.x, box1.min.x),
            f64::min(box0.min.y, box1.min.y),
            f64::min(box0.min.z, box1.min.z),
        );

        let big = Vec3::new(
            f64::max(box0.max.x, box1.max.x),
            f64::max(box0.max.y, box1.max.y),
            f64::max(box0.max.z, box1.max.z),
        );

        Aabb::new(small, big)
    }

    pub fn longest_axis(&self) -> usize {
        let dx = self.max.x - self.min.x;
        let dy = self.max.y - self.min.y;
        let dz = self.max.z - self.min.z;
        if dx > dy && dx > dz {
            0
        } else if dy > dz {
            1
        } else {
            2
        }
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
                min = self.min.x;
                max = self.max.x;
            } else if a == 1 {
                origin = r.origin.y;
                direction = r.direction.y;
                min = self.min.y;
                max = self.max.y;
            } else {
                origin = r.origin.z;
                direction = r.direction.z;
                min = self.min.z;
                max = self.max.z;
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
