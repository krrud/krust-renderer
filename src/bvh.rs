use crate::vec3::Vec3;
use crate::hit::{Object, HittableList, Hittable, BoundingBox};
use crate::hit::HitRecord;
use crate::ray::Ray;
use crate::aabb::Aabb;
use crate::utility::random_int;
use std::cmp::Ordering;
use std::mem;
use crate::material::{Material, Principle};
use crate::sphere::Sphere;
use std::sync::Arc;
use std::thread;
use rayon::prelude::*;


#[derive(Clone)]
pub struct Bvh {
    pub objects: Vec<Arc<Object>>,
    pub time0: f64,
    pub time1: f64,
    pub left: Arc<Object>,
    pub right: Arc<Object>,
    pub bbox: Aabb,
}

impl Bvh { 
    pub fn new(objects: &mut [Arc<Object>], time0: f64, time1: f64) -> Bvh {
        let mut lbox = Aabb::empty();
        let mut rbox = Aabb::empty();
        let mut bbox = Aabb::empty();
        let mut left = Arc::new(Object::empty());
        let mut right = Arc::new(Object::empty());
        
        let axis = random_int(0.0, 2.0);
        let comparator = 
        if axis == 0 {Bvh::box_x_compare} 
        else if axis == 1 {Bvh::box_y_compare}
        else {Bvh::box_z_compare};

        let span = objects.len();
        let mid = span / 2;

        if span <= 0 {
            panic!("Empty BVH");
        } 
        else if span == 1 { 
            left = objects[0].clone();
            right = objects[0].clone();
            bbox = left.bounding_box(0.0,1.0);
        } 
        else if span == 2 {        
            let l = objects[0].clone();
            let r = objects[1].clone();
            bbox = Aabb::surrounding_box(l.bounding_box(0.0,1.0), r.bounding_box(0.0,1.0));
            if comparator(&l, &r) {
                left = l;
                right = r;
            } else {
                left = r;
                right = l;
            }      
        } else {
            objects.sort_by(|a, b| {
                if comparator(a, b) {Ordering::Less}
                else {Ordering::Greater}
            });
            let (left_slice, right_slice) = objects.split_at_mut(mid);
            left = Arc::new(Object::Bvh(Bvh::new(left_slice, time0, time1)));
            right = Arc::new(Object::Bvh(Bvh::new(right_slice, time0, time1)));
            lbox = left.bounding_box(time0, time1);
            rbox = right.bounding_box(time0, time1);
            bbox = Aabb::surrounding_box(lbox, rbox);
        }

        Bvh {
            objects: objects.to_vec(),
            time0, 
            time1,
            left,
            right,
            bbox
        }
    }

    pub fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> (bool, Option<HitRecord>) {
        let bbox_hit = self.bbox.hit(r, t_min, t_max);
        match bbox_hit {
            (false, None) => return (false, None),
            _ => (),
        }        
        let hit_left = self.left.hit(r, t_min, t_max);
        match hit_left { 
            (true, Some(left_hit_rec)) => {
                let hit_right = self.right.hit(r, t_min, left_hit_rec.t);
                match hit_right {
                    (true, Some(right_hit_rec)) => return (true, Some(right_hit_rec)),
                    _ => return (true, Some(left_hit_rec)),
                }
            },
            _ => {
                let hit_right = self.right.hit(r, t_min, t_max);
                match hit_right {
                    (true, Some(right_hit_rec)) => return (true, Some(right_hit_rec)),
                    _ => return (false, None),
                }
            },
        }
    }

    pub fn bounding_box(&self, time0: f64, time1: f64) -> Aabb {
        self.bbox
    }

    pub fn box_compare(a: &Arc<Object>, b: &Arc<Object>, axis: usize) -> bool {
        let box_a = a.bounding_box(0.0, 0.0);
        let box_b = b.bounding_box(0.0, 0.0);
        if axis == 0 {
            return box_a.minimum.x < box_b.minimum.x;
        } else if axis == 1 {
            return box_a.minimum.y < box_b.minimum.y;
        } else {
            return box_a.minimum.z < box_b.minimum.z;
        }
    }
        
    pub fn box_x_compare (a: &Arc<Object>, b: &Arc<Object>) -> bool {
        return Bvh::box_compare(&a, &b, 0);
    }
    
    pub fn box_y_compare (a: &Arc<Object>, b: &Arc<Object>) -> bool {
        return Bvh::box_compare(&a, &b, 1);
    }
    
    pub fn box_z_compare (a: &Arc<Object>, b: &Arc<Object>) -> bool {
        return Bvh::box_compare(&a, &b, 2);
    }
}

