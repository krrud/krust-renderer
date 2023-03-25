use crate::bvh::Bvh;
use crate::camera::Camera;
use crate::ray::Ray;
use crate::vec3::Vec3;
use crate::color::Color;
use crate::aabb::Aabb;
use crate::buffers::{Lobes, FrameBuffers};
use image::{ImageBuffer, Rgb, Rgba, RgbImage, RgbaImage, Rgb32FImage, Rgba32FImage};
use std::io::Write;
use std::{env, fs, thread};
use std::sync::{Arc, Mutex, RwLock};
use crate::utility::{random_float, INF};
use crate::hit::{HitRecord, HittableList, Object, Hittable};
use crate::material::Scatterable;
use std::f64::consts::PI;
use crate::texture::TextureMap;


pub fn ray_color(   
    r: &Ray, world: &Object, depth: u32, 
    max_depth: u32, progressive: bool, 
    skydome: &Option<Arc<TextureMap>>, hide_skydome: bool
    ) -> Lobes {
    if depth <= 0 {
        return Lobes {
            beauty:  Color::black(),
            diffuse: Color::black(),
            specular: Color::black(),
            albedo:  Color::black(), 
            emission:  Color::black(),
        }
    }
    if let (true, Some(hit_rec)) = world.hit(&r, 0.001, INF) {
        if let Some((scattered_ray, albedo, lobe, emit)) = hit_rec.material.scatter(&r, &hit_rec) {
            if let Some(sr) = scattered_ray {
                let emission = if hit_rec.front_face {emit} else {Color::black()};
                let rc = ray_color(&sr, &world, depth - 1, max_depth, progressive, skydome, hide_skydome);
                return Lobes {
                    beauty: emit + (albedo * rc.beauty),
                    diffuse: if lobe == "diffuse" {albedo * rc.beauty} else {Color::black()},
                    specular: if lobe == "specular" {rc.beauty} else if lobe == "metallic" {albedo * rc.beauty} else {Color::black()},
                    albedo, 
                    emission,
                }
            }
        }
    }
    match skydome {
        Some(ref sky) => {
            let unit_direction = Vec3::unit_vector(&r.direction);
            let rotation_degrees: f64 = 45.0;
            let rotation_radians = rotation_degrees.to_radians();
            let phi = unit_direction.z.atan2(unit_direction.x) + rotation_radians;
            let theta = (-unit_direction.y).asin();
            let u = 1.0 - (phi + PI) / (2.0 * PI);
            let v = 1.0 - (theta + PI / 2.0) / PI;        
            let hdr_color = sky.sample(u as f32, v as f32);

            if depth == max_depth && hide_skydome {
                return Lobes {
                    beauty: Color::black(),
                    diffuse: Color::black(),
                    specular: Color::black(),
                    albedo: Color::black(), 
                    emission: Color::black()
            }
            } else {
                return Lobes {
                    beauty: hdr_color,
                    diffuse: Color::black(),
                    specular: Color::black(),
                    albedo: Color::black(), 
                    emission: Color::black(),
                }
            }
  
        },
        None => {

            let unit_direction = Vec3::unit_vector(&r.direction);
            let t = 0.5 * (unit_direction.y() + 1.0);
            let gradient_color = Color::new(0.3, 0.45, 0.1, 0.0);
            let gradient = Color::black() * (1.0 - t) + gradient_color * t;
            return Lobes {
                beauty: Color::new(0.5,0.5,0.5,0.0),
                diffuse: Color::black(),
                specular: Color::black(),
                albedo: Color::black(), 
                emission: Color::black(),
            }
        }
    }
}


pub fn render_pixel(
    x: u32, 
    y: u32,
    height: &u32,
    width: &u32,
    sample: &u16,
    buffer_rgba: &Arc<RwLock<Rgba32FImage>>,
    buffer_diff: &Arc<RwLock<Rgba32FImage>>,
    buffer_spec: &Arc<RwLock<Rgba32FImage>>,
    preview: &Arc<RwLock<RgbaImage>>,
    camera: &Arc<Camera>,
    bvh: &Object,
    depth: u32,
    max_depth: u32,
    progressive: bool,
    skydome: &Option<Arc<TextureMap>>,
    hide_skydome: bool,
    ) -> () {
        
        // get existing rgba vals
        let rgba = buffer_rgba.read().unwrap();
        let pixel = rgba.get_pixel(x, y);
        let previous_rgba =
            Color::new(pixel[0] as f64, pixel[1] as f64, pixel[2] as f64, pixel[3] as f64);
        drop(rgba);

        // get existing diffuse vals
        let diff = buffer_diff.read().unwrap();
        let pixel = diff.get_pixel(x, y);
        let previous_diff =
            Color::new(pixel[0] as f64, pixel[1] as f64, pixel[2] as f64, pixel[3] as f64);
        drop(diff);

        // get existing specular vals
        let spec = buffer_spec.read().unwrap();
        let pixel = spec.get_pixel(x, y);
        let previous_spec =
            Color::new(pixel[0] as f64, pixel[1] as f64, pixel[2] as f64, pixel[3] as f64);
        drop(spec);

        // sample the scene
        let u = (x as f64 + random_float()) / ((width - 1) as f64);
        let v = 1.0 - ((y as f64 + random_float()) / ((height - 1) as f64));
        let r = camera.get_ray(u, v);
        let ray_sample = ray_color(&r, bvh, depth, max_depth, progressive, skydome, hide_skydome);
        let mut rgba_color = ray_sample.beauty;
        let mut diff_color = ray_sample.diffuse;
        let mut spec_color = ray_sample.specular;

        // average in new sample for each lobe
        if sample > &0 {
            let average = (sample + 1) as f64;
            rgba_color = (rgba_color + (previous_rgba * *sample as f64)) /  average;
            diff_color = (diff_color + (previous_diff * *sample as f64)) /  average;
            spec_color = (spec_color + (previous_spec * *sample as f64)) /  average;
        }

        // update rgba buffer
        let mut rgba = buffer_rgba.write().unwrap();
        rgba.put_pixel(x, y, 
            Rgba([
                rgba_color.r as f32, 
                rgba_color.g as f32, 
                rgba_color.b as f32, 
                rgba_color.a as f32
                ]));
        drop(rgba);

        // update diffuse buffer
        let mut diff = buffer_diff.write().unwrap();
        diff.put_pixel(x, y, 
            Rgba([
                diff_color.r as f32, 
                diff_color.g as f32, 
                diff_color.b as f32, 
                diff_color.a as f32
                ]));
        drop(diff);

        // update specular buffer
        let mut spec = buffer_spec.write().unwrap();
        spec.put_pixel(x, y, 
            Rgba([
                spec_color.r as f32, 
                spec_color.g as f32, 
                spec_color.b as f32, 
                spec_color.a as f32
                ]));
        drop(spec);

        // update preview
        let mut preview_buffer = preview.write().unwrap();
        preview_buffer.put_pixel(x,y,
            Rgba([
                (rgba_color.r.sqrt() * 255.999) as u8,
                (rgba_color.g.sqrt() * 255.999) as u8,
                (rgba_color.b.sqrt() * 255.999) as u8,
                (rgba_color.a * 255.999) as u8,
            ]),
        );
        drop(preview_buffer);
}
