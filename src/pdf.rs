use crate::vec3::Vec3;
use crate::onb::Onb;
use crate::lights::QuadLight;
use crate::hit::Object;
use crate::utility::{random_float, INF};
use crate::hit::Hittable;
use crate::ray::Ray;
use std::sync::{Arc, Mutex, RwLock};


pub trait Pdf {
    fn value(&self, direction: &Vec3) -> f64;
    fn generate(& self) -> Vec3;
}

pub struct CosinePdf {
    pub uvw: Onb,
}

impl CosinePdf {
    pub fn new(w: Vec3) -> Self {
        let mut onb = Onb::new();
        onb.build_from_w(w);
        Self { uvw: onb }
    }
}

impl Pdf for CosinePdf {
    fn value(&self, direction: &Vec3) -> f64 {
        let cosine = Vec3::dot(&direction.normalize(), &self.uvw.w());
        if cosine <= 0.0 { 0.0 } else { cosine / std::f64::consts::PI }
    }

    fn generate(& self) -> Vec3 {
        self.uvw.local(Vec3::random_cosine_direction())
    }
}

pub struct LightPdf {
    pub lights: Arc<Vec<Object>>,
    point: Vec3,
    normal: Vec3
}

impl LightPdf{
    pub fn new(lights: Arc<Vec<Object>>, point: Vec3, normal: Vec3) -> LightPdf {
        LightPdf {
            lights,
            point,
            normal
        }
    }
}

impl Pdf for LightPdf {
    fn value(&self, direction: &Vec3) -> f64 {
        let ray = Ray::new(self.point, *direction, 0.0);
        for light in self.lights.iter(){
            if let (true, Some(hit_rec)) = light.hit(&ray, 0.0001, INF) {
                let distance_squared = (hit_rec.point - self.point).length_squared();
                let cosine = self.normal.dot(&direction).max(0.0);
                let mut area = 0.0;
                let mut intensity = 1.0;
                match light {
                    Object::QuadLight(quad_light) => {
                        area = quad_light.area;
                        intensity = quad_light.intensity;
                    }
                    _ => {}
                }
                return distance_squared / (cosine * area);            
            }
        }
    0.0
    }

    fn generate(&self) -> Vec3 {
        let mut to_light = Vec3::black();
        let mut on_light = Vec3::black();
        let mut distance_squared = 0.0;
        let mut light_cosine = 0.0;
        let mut pdf_val = 0.0;
        let mut sum_pdf = 0.0;
        for (i, light) in self.lights.iter().enumerate() {
            match light {
                Object::QuadLight(quad_light) => {
                    let distance_squared = (quad_light.position - self.point).length_squared();
                    sum_pdf += quad_light.area / distance_squared;
                }
                _ => {}
            }
        }

        let mut chosen_light = None;
        for (i, light) in self.lights.iter().enumerate() {
            match light {
                Object::QuadLight(quad_light) => {
                    let distance_squared = (quad_light.position - self.point).length_squared();
                    let pdf = quad_light.area / distance_squared;
                    if chosen_light.is_none() && random_float() < pdf / sum_pdf {
                        chosen_light = Some(quad_light);
                    }
                }
                _ => {}
            }
        }

        // generate scatter direction based on pdf
        if let Some(quad_light) = chosen_light {
            // Generate a random point on the selected light
            let (s, t) = (random_float(), random_float());
            on_light = quad_light.position
                + quad_light.x_axis * (s - 0.5) * quad_light.width
                + quad_light.y_axis * (t - 0.5) * quad_light.height;  

            // Compute the direction to the random point on the light
            to_light = on_light - self.point;
            distance_squared = to_light.length_squared();
            
        }
        to_light.normalize()
    }
}



