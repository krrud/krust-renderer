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
    r: &Ray, world: &Object, lights: &Arc<Vec<Object>>, depth: u32, 
    max_depth: u32, progressive: bool, 
    skydome: &Option<Arc<TextureMap>>, hide_skydome: bool
    ) -> Lobes {
    if depth <= 0 {
        return Lobes::empty();
    }

    if let (true, Some(hit_rec)) = world.hit(&r, 0.0001, INF) {
        if let Some((scattered_ray, albedo, lobe, emit)) = hit_rec.material.scatter(&r, &hit_rec, lights) {
            if let Some(ray) = scattered_ray {
                // sample scene
                let sample = ray_color(&ray, &world, &lights, depth - 1, max_depth, progressive, skydome, hide_skydome);
                let composite = emit + albedo * sample.beauty;

                // sort lobes
                let mut color = Lobes::empty();
                color.beauty = composite;
                color.diffuse = if lobe == "diffuse" {composite} else {Color::black()};
                color.specular = 
                if lobe == "specular" {composite}
                else if lobe == "metallic" {composite}
                else {Color::black()};
                color.emission = if hit_rec.front_face {emit} else {Color::black()};

                // directional lights
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

                    let view_dir = -(r.direction).unit_vector();
                    let dir_light = DirectionalLight::new(Vec3::new(-0.4494639595455351, 0.6829063708571647, -0.5758654684145821), Color::white(), 0.5);
                    let dir_light_contrib = dir_light.irradiance(hit_rec.normal, view_dir, roughness, &lobe);
    
                    // if !dir_light.shadow(&hit_rec.point, &world){
                    //     if lobe == "diffuse" {
                    //         color.beauty = color.beauty + (albedo * dir_light_contrib * diffuse_weight);
                    //         color.diffuse = color.diffuse + (albedo * dir_light_contrib * diffuse_weight);
                    //     } else if lobe == "specular" {
                    //         color.beauty = color.beauty + (dir_light_contrib * specular_weight);
                    //         color.specular = color.specular + (dir_light_contrib * specular_weight);
                    //     }                        
                    // }                    
                }
                
                // return final composite
                if color.beauty.sum() < 0.001 && color.emission.sum() < 0.001 {
                    return Lobes::empty();
                } else if color.beauty.has_nan() {
                    return Lobes::empty();
                } else {
                    return color
                }                
            }
        }
    }
    match skydome {
        Some(ref sky) => {
            let unit_direction = Vec3::unit_vector(&r.direction);
            let rotation_degrees: f64 = 0.0;
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
            let gradient_color = Color::new(0.63, 0.75, 1.0, if hide_skydome {0.0} else {1.0});
            let gradient = Color::new(1.0, 1.0, 1.0, if hide_skydome {0.0} else {1.0}) * (1.0 - t) + gradient_color * t;
            return Lobes {
                beauty: Color::black(),
                diffuse: Color::black(),
                specular: Color::black(),
                albedo: Color::black(), 
                emission: Color::black(),
            }
        }
    }
}


// pub fn render_pixel(
//     x: u32, 
//     y: u32,
//     height: &u32,
//     width: &u32,
//     sample: &u16,
//     buffer_rgba: &Arc<RwLock<Rgba32FImage>>,
//     buffer_diff: &Arc<RwLock<Rgba32FImage>>,
//     buffer_spec: &Arc<RwLock<Rgba32FImage>>,
//     preview: &Arc<RwLock<RgbaImage>>,
//     camera: &Arc<Camera>,
//     bvh: &Object,
//     depth: u32,
//     max_depth: u32,
//     progressive: bool,
//     skydome: &Option<Arc<TextureMap>>,
//     hide_skydome: bool,
//     ) -> () {
        
//         // get existing rgba vals
//         let rgba = buffer_rgba.read().unwrap();
//         let pixel = rgba.get_pixel(x, y);
//         let previous_rgba =
//             Color::new(pixel[0] as f64, pixel[1] as f64, pixel[2] as f64, pixel[3] as f64);
//         drop(rgba);

//         // get existing diffuse vals
//         let diff = buffer_diff.read().unwrap();
//         let pixel = diff.get_pixel(x, y);
//         let previous_diff =
//             Color::new(pixel[0] as f64, pixel[1] as f64, pixel[2] as f64, pixel[3] as f64);
//         drop(diff);

//         // get existing specular vals
//         let spec = buffer_spec.read().unwrap();
//         let pixel = spec.get_pixel(x, y);
//         let previous_spec =
//             Color::new(pixel[0] as f64, pixel[1] as f64, pixel[2] as f64, pixel[3] as f64);
//         drop(spec);

//         // sample the scene
//         let u = (x as f64 + random_float()) / ((width - 1) as f64);
//         let v = 1.0 - ((y as f64 + random_float()) / ((height - 1) as f64));
//         let r = camera.get_ray(u, v);
//         let ray_sample = ray_color(&r, bvh, depth, max_depth, progressive, skydome, hide_skydome);
//         let mut rgba_color = ray_sample.beauty;
//         let mut diff_color = ray_sample.diffuse;
//         let mut spec_color = ray_sample.specular;

//         // average in new sample for each lobe
//         if sample > &0 {
//             let average = (sample + 1) as f64;
//             rgba_color = (rgba_color + (previous_rgba * *sample as f64)) /  average;
//             diff_color = (diff_color + (previous_diff * *sample as f64)) /  average;
//             spec_color = (spec_color + (previous_spec * *sample as f64)) /  average;
//         }

//         // update rgba buffer
//         let mut rgba = buffer_rgba.write().unwrap();
//         rgba.put_pixel(x, y, 
//             Rgba([
//                 rgba_color.r as f32, 
//                 rgba_color.g as f32, 
//                 rgba_color.b as f32, 
//                 rgba_color.a as f32
//                 ]));
//         drop(rgba);

//         // update diffuse buffer
//         let mut diff = buffer_diff.write().unwrap();
//         diff.put_pixel(x, y, 
//             Rgba([
//                 diff_color.r as f32, 
//                 diff_color.g as f32, 
//                 diff_color.b as f32, 
//                 diff_color.a as f32
//                 ]));
//         drop(diff);

//         // update specular buffer
//         let mut spec = buffer_spec.write().unwrap();
//         spec.put_pixel(x, y, 
//             Rgba([
//                 spec_color.r as f32, 
//                 spec_color.g as f32, 
//                 spec_color.b as f32, 
//                 spec_color.a as f32
//                 ]));
//         drop(spec);

//         // update preview
//         let mut preview_buffer = preview.write().unwrap();
//         preview_buffer.put_pixel(x,y,
//             Rgba([
//                 (rgba_color.r.sqrt() * 255.999) as u8,
//                 (rgba_color.g.sqrt() * 255.999) as u8,
//                 (rgba_color.b.sqrt() * 255.999) as u8,
//                 (rgba_color.a * 255.999) as u8,
//             ]),
//         );
//         drop(preview_buffer);
// }

pub fn render_chunk(
    pixel_chunks: &Vec<(u32, u32)>,
    height: u32,
    width: u32,
    sample: &u16,
    subsamples: u32,
    camera: &Arc<Camera>,
    bvh: &Object,
    lights: &Arc<Vec<Object>>,
    depth: u32,
    max_depth: u32,
    progressive: bool,
    skydome: &Option<Arc<TextureMap>>,
    hide_skydome: bool,
    ) -> Vec<(u32, u32, Lobes)> {
        let mut pixel_colors = Vec::new();
        for pixel in pixel_chunks {
            let (x, y) = pixel;
            let mut sum = Lobes::empty();
            for i in 0..subsamples{                
                let u = (*x as f64 + random_float()) / ((width - 1) as f64);
                let v = 1.0 - ((*y as f64 + random_float()) / ((height - 1) as f64));
                let r = camera.get_ray(u, v);
                let sample = ray_color(&r, bvh, lights, depth, max_depth, progressive, skydome, hide_skydome);
                sum = sum + sample;
            }
            let color = sum.average_samples(subsamples);
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
