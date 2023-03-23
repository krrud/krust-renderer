use crate::vec3::Vec3;
use crate::vec2::Vec2;
use crate::hit::HitRecord;
use crate::material::Material;
use crate::ray::Ray;
use crate::tri::Tri;
use crate::aabb::Aabb;

#[derive(Debug, Clone)]
pub struct TriMesh {
    pub tris: Vec<Tri>,
    pub material: Material,
}

impl TriMesh {
    pub fn new(tris: Vec<Tri>, material: Material) -> TriMesh {
        TriMesh {
            tris,
            material,
        }
    }

    pub fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        for tri in self.tris.iter() {
            const EPSILON: f64 = 0.0000001;
            let (v0, v1, v2) = (tri.vertices[0], tri.vertices[1], tri.vertices[2]);
            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            let h = r.direction.cross(&edge2);
            let a = edge1.dot(&h);
            if a > -EPSILON && a < EPSILON {
                return None;
            }
            let f = 1.0 / a;
            let s = r.origin - v0;
            let u = f * s.dot(&h);
            if u < 0.0 || u > 1.0 {
                return None;
            }
            let q = s.cross(&edge1);
            let v = f * r.direction.dot(&q);
            if v < 0.0 || u + v > 1.0 {
                return None;
            }
            let rt = f * edge2.dot(&q);
            if rt > EPSILON {
                let p = r.at(rt);
                let normal = edge1.cross(&edge2);
                let front_face = true;
                return Some(HitRecord {
                    t: rt,
                    point: p,
                    normal: if front_face { normal } else { -normal },
                    uv: Vec2::zero(),
                    front_face,
                    material: self.material,
                });                
            }
        }
        None
    }

    pub fn bounding_box(&self, time0: f64, time1: f64) -> Aabb {
        Aabb::new(Vec3::black(), Vec3::black())
    }
}
