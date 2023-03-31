use crate::hit::{Object, Hittable};
use crate::vec3::Vec3;
use crate::vec2::Vec2;
use crate::color::Color;
use crate::ray::Ray;
use crate::utility::{random_float, INF};
use std::f64::consts::PI;
use cgmath::{Deg, Angle};
use nalgebra::{Rotation3, Vector3};
use crate::tri::Tri;
use crate::material::{Emits, Light, Material, Principle, Scatterable};
use std::sync::Arc;
use crate::aabb::Aabb;
use crate::hit::BoundingBox;
use crate::HitRecord;


#[derive(Clone)]
pub struct QuadLight {
    pub color: Color,
    pub intensity: f64,
    pub power: f64,
    pub tris: Vec<Object>,
    pub vertices: Vec<Vec3>,
    pub position: Vec3,
    pub area: f64,
    pub normal: Vec3,
    pub x_axis: Vec3,
    pub y_axis: Vec3,
    pub width: f64,
    pub height: f64
}

impl QuadLight {
    pub fn new(color: Color, intensity: f64, vertices: Vec<Vec3>) -> Self {
        let x_axis = (vertices[1] - vertices[0]).normalize();
        let y_axis = x_axis.cross(&(vertices[3] - vertices[0])).normalize();
        let width = (vertices[1] - vertices[0]).length();
        let height = (vertices[3] - vertices[0]).length();

        let position = (vertices[0] + vertices[1] + vertices[2] + vertices[3]) / 4.0;       
        let material = Arc::new(Material::Light(Light::new(color, intensity)));
        let v1 = vec![vertices[0], vertices[1], vertices[2]];
        let v2 = vec![vertices[2], vertices[3], vertices[0]];
        let normals = vec![Vec3::black(); 3];
        let uvs = vec![Vec2::zero(); 3];
        let tri1 = Tri::new(v1, normals.clone(), uvs.clone(), material.clone(), false);
        let tri2 = Tri::new(v2, normals, uvs, material.clone(), false);
        let area = tri1.area + tri2.area;
        let normal = Vec3::cross(&(vertices[1]-vertices[0]), &(vertices[2]-vertices[1]));
        let tris = vec![Object::Tri(tri1), Object::Tri(tri2)];

        let luminance = 0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b;
        let power = area * intensity * luminance;

        Self { 
            color, 
            intensity, 
            power,
            tris,
            vertices,
            position,
            area,
            normal,
            x_axis,
            y_axis,
            width,
            height
         }
    }

    pub fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> (bool, Option<HitRecord>) {
        for tri in self.tris.iter() {
            if let (true, Some(hit_rec)) = tri.hit(&r, t_min, t_max){
                return (true, Some(hit_rec));
            }
        }
        (false, None) 
    }

    pub fn bounding_box(&self, time0: f64, time1: f64) -> Aabb {
        Aabb::surrounding_box(self.tris[0].bounding_box(0.0, 1.0), self.tris[1].bounding_box(0.0, 1.0))

    }

    pub fn radius(&self) -> f64 {
        let diagonal = (self.vertices[0] - self.vertices[2]).length();
        diagonal / 2.0
    }

    pub fn pdf_value(&self, origin: &Vec3, v: &Vec3) -> f64 {
        let ray = Ray::new(*origin, *v, 0.0);
        let (hit, opt_rec) = self.hit(&ray, 0.001, f64::INFINITY);
        if !hit {
            return 0.0;
        }
        let rec = opt_rec.unwrap();
        let distance_squared = rec.t * rec.t * v.length_squared();
        let cosine = f64::abs(Vec3::dot(v, &rec.normal) / v.length());
        distance_squared / (cosine * self.area)
    }   

}


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

        let halfway = (-self.direction + view_dir).normalize();
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