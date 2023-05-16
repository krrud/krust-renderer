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
use std::f64::consts::PI;
use crate::texture::TextureMap;
use crate::lights::DirectionalLight;
use crate::material::{Emits, Light, Material, Principle, Scatterable};


pub fn ray_color(   
    r: &Ray, 
    world: &Object, 
    quad_lights: &Arc<Vec<Object>>, 
    dir_lights: &Arc<Vec<DirectionalLight>>, 
    depth: u32, 
    max_depth: u32, 
    progressive: bool, 
    skydome: &Option<Arc<TextureMap>>,
    hide_skydome: bool
    ) -> Lobes {

    if depth <= 0 {
        return Lobes::empty();
    }

    if let (true, Some(hit_rec)) = world.hit(&r, 0.0001, INF) {
        if let Some((ray, albedo, emission, lobe)) = hit_rec.material.scatter(&r, &hit_rec, quad_lights) {
            // sample scene
            let sample = ray_color(&ray, &world, &quad_lights, &dir_lights, depth - 1, max_depth, progressive, skydome, hide_skydome);
            let emit = if hit_rec.front_face {emission} else {Color::black()};
            let composite = emit + albedo * sample.rgba;

            // sort lobes
            let mut color = Lobes::empty();
            color.rgba = composite;
            color.diffuse = if lobe == "diffuse" {composite} else {Color::black()};
            color.specular = 
            if lobe == "specular" {composite}
            else {Color::black()};
            color.emission = emission;

            // material properties
            let mut diffuse_weight = 0.0;
            let mut specular_weight = 0.0;
            let mut roughness = 0.0;
            
            if let Material::Principle(principle) = &*hit_rec.material {
                diffuse_weight = principle.diffuse_weight;
                if let Some(dwt) = &principle.diffuse_weight_texture {
                    diffuse_weight = principle.diffuse_weight_texture
                        .as_ref()
                        .map(|t| t.sample(hit_rec.uv.x, hit_rec.uv.y))
                        .unwrap_or_else(|| Color::new(0.0, 1.0, 1.0, 1.0)).r;
                } 
                specular_weight = principle.specular_weight;
                if let Some(rt) = &principle.specular_weight_texture {
                    specular_weight = principle.specular_weight_texture
                        .as_ref()
                        .map(|t| t.sample(hit_rec.uv.x, hit_rec.uv.y))
                        .unwrap_or_else(|| Color::new(0.0, 1.0, 1.0, 1.0)).r;
                } 
                roughness = principle.roughness;
                if let Some(rt) = &principle.roughness_texture {
                    roughness = principle.roughness_texture
                        .as_ref()
                        .map(|t| t.sample(hit_rec.uv.x, hit_rec.uv.y))
                        .unwrap_or_else(|| Color::new(0.0, 1.0, 1.0, 1.0)).r;
                } 
                roughness = (1.0 - roughness).powf(4.0) * 1000.0 + 3.5;

                // directional lights
                let view_dir = -(r.direction).normalize();
                for dir_light in dir_lights.iter() {
                    let contrib = dir_light.irradiance(hit_rec.normal, view_dir, roughness, &lobe);
                    if !dir_light.shadow(&hit_rec.point, &world){
                        if lobe == "diffuse" {
                            color.rgba = color.rgba + (albedo * contrib * diffuse_weight);
                            color.diffuse = color.diffuse + (albedo * contrib * diffuse_weight);
                        } else if lobe == "specular" {
                            color.rgba = color.rgba + (contrib * specular_weight);
                            color.specular = color.specular + (contrib * specular_weight);
                        }                        
                    }   
                }                   
            }
            
            // cull and clip
            if color.rgba.sum() < 0.001 && color.emission.sum() < 0.001 {
                return Lobes::empty()
            } else if color.rgba.max() > 80.0 && color.emission.sum() < 0.001 {
                return Lobes::empty()
            } else if color.rgba.has_nan() {
                return Lobes::empty()
            } else {
                return color
            }                 
        }
    }
    match skydome {
        Some(ref sky) => {
            let unit_direction = Vec3::normalize(&r.direction);
            let rotation_degrees: f64 = 60.0;// crab rotation 60.0
            let rotation_radians = rotation_degrees.to_radians();
            let phi = unit_direction.z.atan2(unit_direction.x) + rotation_radians;
            let theta = (-unit_direction.y).asin();
            let u = 1.0 - (phi + PI) / (2.0 * PI);
            let v = 1.0 - (theta + PI / 2.0) / PI;        
            let mut sky_color = sky.sample(u as f32, v as f32);

            if depth == max_depth && hide_skydome {
                return Lobes::empty()
            } else {
                return Lobes {
                    rgba: sky_color,
                    diffuse: Color::black(),
                    specular: Color::black(), 
                    emission: Color::black(),
                }
            }
  
        },
        None => {

            let unit_direction = Vec3::normalize(&r.direction);
            let t = 0.5 * (unit_direction.y() + 1.0);
            let gradient_color = Color::new(0.63, 0.75, 1.0, if hide_skydome {0.0} else {1.0});
            let gradient = Color::new(1.0, 1.0, 1.0, if hide_skydome {0.0} else {1.0}) * (1.0 - t) + gradient_color * t;
            return Lobes {
                rgba: Color::black(),// gradient_color*gradient_color,
                diffuse: Color::black(),
                specular: Color::black(),
                emission: Color::black(),
            }
        }
    }
}

pub fn render_chunk(
    pixel_chunks: &Vec<(u32, u32)>,
    height: u32,
    width: u32,
    sample: &u16,
    camera: &Arc<Camera>,
    bvh: &Object,
    quad_lights: &Arc<Vec<Object>>,
    dir_lights: &Arc<Vec<DirectionalLight>>,
    depth: u32,
    max_depth: u32,
    progressive: bool,
    skydome: &Option<Arc<TextureMap>>,
    hide_skydome: bool,
    ) -> Vec<(u32, u32, Lobes)> {
        let mut pixel_colors = Vec::new();
        for pixel in pixel_chunks {
            let (x, y) = pixel;               
            let u = (*x as f64 + random_float()) / ((width - 1) as f64);
            let v = 1.0 - ((*y as f64 + random_float()) / ((height - 1) as f64));
            let r = camera.get_ray(u, v);
            let color = ray_color(&r, bvh, quad_lights, dir_lights, depth, max_depth, progressive, skydome, hide_skydome);
            pixel_colors.push((*x, *y, color));
        }
        pixel_colors
}

pub fn get_pixel_chunks(chunk_size: usize, width: usize, height: usize) -> Vec<Vec<(u32, u32)>> {

    let mut chunks = Vec::new();
    for y in (0..height).step_by(chunk_size as usize) {
        for x in (0..width).step_by(chunk_size as usize) {
            let chunk_width = if x + chunk_size > width {
                width - x
            } else {
                chunk_size
            };
            let chunk_height = if y + chunk_size > height {
                height - y
            } else {
                chunk_size
            };

            let mut chunk = Vec::new();
            for i in 0..chunk_width {
                for j in 0..chunk_height {
                    chunk.push(((x + i) as u32, (y + j) as u32));
                }
            }
            chunks.push(chunk);
        }
    }
    chunks
}
