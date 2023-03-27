use crate::hit::{Object, Hittable};
use crate::vec3::Vec3;
use crate::color::Color;
use crate::ray::Ray;
use crate::utility::{random_float, INF};
use std::f64::consts::PI;
use cgmath::{Deg, Angle};
use nalgebra::{Rotation3, Vector3};


pub struct DirectionalLight {
    direction: Vec3,
    color: Color,
    softness: f64,
}

impl DirectionalLight {
    pub fn new(direction: Vec3, color: Color, softness: f64) -> Self {
        let direction = Vec3::new(-direction.x, -direction.y, -direction.z);
        Self { direction, color, softness }
    }

    pub fn irradiance(&self, normal: Vec3, view_dir: Vec3, roughness: f64, lobe: &str) -> Color {
        let cos_theta = (-self.direction).dot(&normal).max(0.0);
        let diffuse = self.color * cos_theta;

        let halfway = (-self.direction + view_dir).unit_vector();
        let cos_alpha = normal.dot(&halfway).max(0.0);
        let specular = self.color * 5.0 * f64::powf(cos_alpha, roughness);
        
        if lobe == "diffuse" { return diffuse }
        else if lobe == "specular" { return specular }
        else { return Color::black() }
        
    }

    pub fn shadow(&self, hit_point: &Vec3, world: &Object) -> bool {
        let shadow_direction = -self.direction;
        let shadow_origin = *hit_point + shadow_direction * 0.001;
        let soft = Vec3::random_unit_vector() * self.softness / 10.0;
        let ray = Ray::new(shadow_origin, shadow_direction + soft, 0.0);
        if let (true, Some(hit_rec)) = world.hit(&ray, 0.001, INF) {
            return true;

        }
        false
    }
}

fn schlick_specular(view_dir: Vec3, light_dir: Vec3, normal: Vec3, roughness: f64) -> f64 {
    let r0 = (1.0 - roughness) / (1.0 + roughness);
    let r0 = r0 * r0;
    let cos_theta = view_dir.dot(&normal);
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
    let cos_alpha = light_dir.dot(&normal);
    let sin_alpha = (1.0 - cos_alpha * cos_alpha).sqrt();
    let a = (roughness + 1.0) * (roughness + 1.0) / 8.0;
    let f = a + (1.0 - a) * (1.0 - cos_alpha).powf(5.0);
    let d = d_ggx(roughness, cos_alpha);
    let g = g_ggx(roughness, cos_theta, sin_theta, cos_alpha, sin_alpha);
    let specular = f * g * d / (4.0 * cos_theta * cos_alpha);
    specular * r0.max(1.0 - cos_alpha).powf(5.0)
}

fn d_ggx(roughness: f64, cos_alpha: f64) -> f64 {
    let alpha = roughness * roughness;
    let alpha2 = alpha * alpha;
    (alpha2 - 1.0) / (PI * alpha2 * cos_alpha.powf(4.0) + 0.001)
}

fn g_ggx(roughness: f64, cos_theta: f64, sin_theta: f64, cos_alpha: f64, sin_alpha: f64) -> f64 {
    let alpha = roughness * roughness;
    let a = alpha / (1.0 - cos_alpha);
    let k = a * 0.5;
    let g1 = 1.0 / (1.0 + k * k);
    let g2 = g1 * (1.0 + (cos_theta / sin_theta).powf(2.0));
    g1 * g2
}