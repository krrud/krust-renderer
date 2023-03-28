use crate::vec3::Vec3;
use crate::vec2::Vec2;
use crate::ray::Ray;
use crate::sphere::Sphere;
use crate::tri::Tri;
// use crate::trimesh::TriMesh;
use crate::material::{Material, Principle};
use crate::aabb::Aabb;
use crate::bvh::Bvh;
use std::sync::Arc;
use crate::lights::QuadLight;


#[derive(Clone)]
pub enum Object{
    Sphere(Sphere),
    Tri(Tri),
    QuadLight(QuadLight),
    // TriMesh(TriMesh),
    Aabb(Aabb),
    Bvh(Bvh),
    HittableList(HittableList),
}

impl Object{
    pub fn empty() -> Object  {
        let mat = Arc::new(Material::Principle(Principle::default()));
        Object::Sphere(Sphere::new(Vec3::black(), Vec3::black(), 0.0, 1.0, 0.001, mat))
    }
}

pub trait Hittable {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> (bool, Option<HitRecord>);

    fn pdf_value(&self, o: &Vec3, v: &Vec3) -> f64 {
        0.0
    }

    fn random(&self, o: &Vec3) -> Vec3 {
        Vec3::new(1.0, 0.0, 0.0)
    }
}

impl Hittable for Object {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> (bool, Option<HitRecord>) {
        match self {
            Object::Sphere(sphere) => sphere.hit(ray, t_min, t_max),
            Object::Tri(tri) => tri.hit(ray, t_min, t_max),
            Object::QuadLight(ql) => ql.hit(ray, t_min, t_max),
            // Object::TriMesh(trimesh) => trimesh.hit(ray, t_min, t_max),
            Object::Aabb(aabb) => aabb.hit(ray, t_min, t_max),
            Object::Bvh(bvh) => bvh.hit(ray, t_min, t_max),
            Object::HittableList(hl) => hl.hit(ray, t_min, t_max),
        }
    }

    fn pdf_value(&self, o: &Vec3, v: &Vec3) -> f64 {
        match self {
            Object::QuadLight(ql) => ql.pdf_value(o, v),
            _=> {0.0}
        }
    }
}

pub trait BoundingBox {
    fn bounding_box(&self, time0: f64, time1: f64) -> Aabb;
}

impl BoundingBox for Object {
    fn bounding_box(&self, time0: f64, time1: f64) -> Aabb {
        match self {
            Object::Sphere(sphere) => sphere.bounding_box(time0, time1),
            Object::Tri(tri) => tri.bounding_box(time0, time1),
            Object::QuadLight(ql) => ql.bounding_box(time0, time1),
            // Object::TriMesh(trimesh) => trimesh.bounding_box(time0, time1),
            Object::Aabb(aabb) => aabb.bounding_box(time0, time1),
            Object::Bvh(bvh) => bvh.bounding_box(time0, time1),
            Object::HittableList(hl) => hl.bounding_box(time0, time1),
        }
    }
}

pub struct HitRecord {
    pub t: f64,
    pub point: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub front_face: bool,
    pub material: Arc<Material>,
}

impl HitRecord {
    pub fn hit_world(world: &[Object], r: &Ray, t_min: f64, t_max: f64) -> (bool, Option<HitRecord>) {
        let mut t_nearest = t_max;
        let mut hit_record = (false, None);
        for obj in world {
            if let (bool, Some(hit)) = obj.hit(r, t_min, t_nearest) {
                t_nearest = hit.t;
                hit_record = (true, Some(hit));
            }
        }
        hit_record
    }
}
#[derive(Clone)]
pub struct HittableList {
    pub objects: Vec<Arc<Object>>
}

impl HittableList {
    pub fn new() -> HittableList {
        HittableList{objects: Vec::new()}
    }


    pub fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> (bool, Option<HitRecord>) {
        let mut t_nearest = t_max;
        let mut hit_record = (false, None);
        for obj in &self.objects {
            if let (bool, Some(hit)) = obj.hit(r, t_min, t_nearest) {
                t_nearest = hit.t;
                hit_record = (true, Some(hit));
            }
        }
        hit_record
    }

    pub fn bounding_box(&self, time0: f64, time1: f64) ->Aabb {  
        let mut output_box: Aabb = Aabb::new(Vec3::black()*0.0001, Vec3::black()*0.001);
        for object in &self.objects {
            output_box = Aabb::surrounding_box(output_box, object.bounding_box(time0, time1));
        }
        output_box
    }

    pub fn new_from_vec(obj_vec: Vec<Arc<Object>>) -> HittableList {
        let mut list = Vec::new();
        for obj in obj_vec {
            list.push(obj);
        }
        HittableList{ objects: list }
    }

}



