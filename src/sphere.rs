
use crate::vec3::Vec3;
use crate::vec2::Vec2;
use crate::hit::Hittable;
use crate::hit::HitRecord;
use crate::ray::Ray;
use crate::material::Material;
use crate::aabb::Aabb;
use std::sync::Arc;
use std::f64::consts::PI;


#[derive(Debug, Clone)]
pub struct Sphere {
    pub center0: Vec3,
    pub center1: Vec3,
    pub time0: f64,
    pub time1: f64,
    pub radius: f64,
    pub area: f64,
    pub material: Arc<Material>,
}

impl Sphere {
    pub fn new(center0: Vec3, center1: Vec3, time0: f64, time1: f64, radius: f64, material: Arc<Material>) -> Sphere {
        let area = 4.0 * PI * radius.powf(2.0);
        Sphere {
            center0,
            center1,
            time0,
            time1,
            radius,
            area,
            material,
        }
    }

    pub fn center(&self, time: f64) -> Vec3 {
        self.center0 + (self.center1 - self.center0) * ((time - self.time0) / (self.time1 - self.time0))
    }

    pub fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> (bool, Option<HitRecord>) {
        let oc: Vec3 = r.origin - self.center(r.time);
        let a = &r.direction.length_squared();
        let half_b = Vec3::dot(&oc, &r.direction);
        let c = &oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;
        if discriminant >= 0.0 {
            let sqrtd = discriminant.sqrt();
            let root_a = ((-half_b) - sqrtd) / a;
            let root_b = ((-half_b) + sqrtd) / a;
            for root in [root_a, root_b].iter() {
                if *root < t_max && *root > t_min {
                    let p = r.at(*root);
                    let normal = ((p - self.center(r.time)) / self.radius).normalize();
                    let front_face = r.direction.dot(&normal) < 0.0;

                    let theta = f64::acos(-p.y);
                    let phi = f64::atan2(-p.z, p.x) + PI;        
                    let u = phi / (2.0*PI);
                    let v = theta / PI;
                    return (true,
                        Some(HitRecord {
                        t: *root,
                        point: p,
                        normal: if front_face {normal} else {-normal},
                        uv: Vec2::new(u as f32, v as f32),
                        front_face,
                        material: self.material.clone(), 
                    }));
                }
            }
        }
        (false, None)  
    }

    pub fn bounding_box(&self, time0: f64, time1: f64) -> Aabb {
        let box0 = Aabb::new(
            self.center(time0) - Vec3::new(self.radius, self.radius, self.radius),
            self.center(time0) + Vec3::new(self.radius, self.radius, self.radius));
        let box1 = Aabb::new(
            self.center(time1) - Vec3::new(self.radius, self.radius, self.radius),
            self.center(time1) + Vec3::new(self.radius, self.radius, self.radius));
        Aabb::surrounding_box(box0, box1)
    }
}

