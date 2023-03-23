use crate::hit::HitRecord;
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Vec3;
use crate::vec2::Vec2;
use crate::aabb::Aabb;
use crate::color::Color;
use crate::texture::TextureMap;
use std::sync::Arc;


#[derive(Debug, Clone)]
pub struct Tri {
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub material: Arc<Material>,
    pub smooth: bool,
}

impl Tri {
    pub fn new(vertices: Vec<Vec3>, normals: Vec<Vec3>, uvs: Vec<Vec2>, material: Arc<Material>, smooth: bool) -> Tri {
        Tri {
            vertices,
            normals,
            uvs,
            material,
            smooth,
        }
    }

    pub fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> (bool, Option<HitRecord>) {
        const EPSILON: f64 = 0.0000001;
        let (v0, v1, v2) = (self.vertices[0], self.vertices[1], self.vertices[2]);
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let h = r.direction.cross(&edge2);
        let a = edge1.dot(&h);
        if a > -EPSILON && a < EPSILON {
            return (false, None);
        }
        let f = 1.0 / a;
        let s = r.origin - v0;
        let u = f * s.dot(&h);
        if u < 0.0 || u > 1.0 {
            return (false, None);
        }
        let q = s.cross(&edge1);
        let v = f * r.direction.dot(&q);
        if v < 0.0 || u + v > 1.0 {
            return (false, None);
        }
        let t = f * edge2.dot(&q);
        if t > EPSILON {
            if t < t_max && t > t_min {
                let p = r.at(t);
                let mut normal: Vec3;
                let uv = (self.uvs[0] * (1.0 - u - v)) + (self.uvs[1] * u) + (self.uvs[2] * v);
                if self.smooth {
                    normal = ((self.normals[0] * (1.0-u-v)) + (self.normals[1] * u) + (self.normals[2] * v)).unit_vector();
                } else {
                    normal = (&edge1).cross(&edge2).unit_vector();
                }
                normal = (&edge1).cross(&edge2).unit_vector();
                let front_face = normal.dot(&r.direction) < 0.0;
                return (true,
                Some(HitRecord {
                t,
                point: p,
                normal: if front_face {normal} else {-normal},
                uv,
                front_face,
                material: self.material.clone(),
                }));                
            }
        } else {
            return (false, None);
        }
        (false, None)
    }

    pub fn bounding_box(&self, time0: f64, time1: f64) -> Aabb {
        let pad = 0.001;
        let (v0, v1, v2) = (self.vertices[0], self.vertices[1], self.vertices[2]);
        let min_x = f64::min(v0.x - pad, f64::min(v1.x - pad, v2.x - pad));
        let min_y = f64::min(v0.y - pad, f64::min(v1.y - pad, v2.y - pad));
        let min_z = f64::min(v0.z - pad, f64::min(v1.z - pad, v2.z - pad));
        let max_x = f64::max(v0.x + pad, f64::max(v1.x + pad, v2.x + pad));
        let max_y = f64::max(v0.y + pad, f64::max(v1.y + pad, v2.y + pad));
        let max_z = f64::max(v0.z + pad, f64::max(v1.z + pad, v2.z + pad));
        let min = Vec3::new(min_x, min_y, min_z);
        let max = Vec3::new(max_x, max_y, max_z);
        Aabb::new(min, max)
    }

    fn get_texture_color2(&self, point: &Vec3) -> Color {
        let (v0, v1, v2) = (self.vertices[0], self.vertices[1], self.vertices[2]);
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let v = *point - v0;
        let u = (v.dot(&edge2) * edge1.length_squared() - v.dot(&edge1) * edge2.dot(&edge1)) / (edge1.length_squared() * edge2.length_squared() - edge1.dot(&edge2).powi(2));
        let v = (v.dot(&edge1) * edge2.length_squared() - v.dot(&edge2) * edge1.dot(&edge2)) / (edge1.length_squared() * edge2.length_squared() - edge1.dot(&edge2).powi(2));
        let w = 1.0 - u - v;
    
        let uv0 = self.uvs[0];
        let uv1 = self.uvs[1];
        let uv2 = self.uvs[2];
        let tex_coord = uv0 * w + uv1 * u + uv2 * v;
    
        let texture = TextureMap::new("texture_test.jpg");
        texture.sample(tex_coord.x, tex_coord.y)
    }


}

