use crate::vec3::Vec3;
use crate::ray::Ray;
use crate::utility::{degrees_to_radians, random_range};

pub struct Camera {
    pub fov: f64,
    pub aspect_ratio: f64,
    pub aperature: f64,
    pub origin: Vec3,
    pub aim: Vec3,
    pub focus: Vec3,
    pub time0: f64,
    pub time1: f64,
    horizontal: Vec3,
    vertical: Vec3,
    lower_left_corner: Vec3,
    u: Vec3,
    v: Vec3,
    
}

impl Camera {
    pub fn new(
        fov: f64,
        aspect_ratio: f64,
        aperature: f64,
        origin: Vec3,
        aim: Vec3,
        focus: Vec3,
        time0: f64,
        time1: f64,
    ) -> Camera {
        let focus_distance = (origin-focus).length();
        let vup = Vec3::new(0.0,1.0,0.0);
        let theta = degrees_to_radians(fov);
        let h = f64::tan(theta/2.0);
        let viewport_height: f64 = 2.0 * h;
        let viewport_width: f64 = aspect_ratio * viewport_height;

        let w = Vec3::unit_vector(&(origin - aim));
        let u = Vec3::unit_vector(&Vec3::cross(&vup, &w));
        let v = Vec3::cross(&w, &u);

        let horizontal = u * viewport_width * focus_distance;
        let vertical  = v * viewport_height * focus_distance;
        let lower_left_corner =
        origin - horizontal / 2.0 - vertical / 2.0 - w * focus_distance;

        Camera{
            fov,
            aspect_ratio,
            aperature,
            origin,
            aim,
            focus,
            time0, 
            time1,
            horizontal,
            vertical,
            lower_left_corner,
            u,
            v,
        }
    }

    pub fn get_ray(&self, s: f64, t: f64) -> Ray {
        let rd = Vec3::random_in_unit_disk() * (self.aperature / 2.0);
        let offset = self.u * rd.x() + self.v * rd.y();
        Ray::new(
            self.origin + offset, 
            self.lower_left_corner + self.horizontal*s + self.vertical*t - self.origin - offset,
            random_range(self.time0, self.time1),
        )
    }
}

