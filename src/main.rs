mod camera;
mod hit;
mod material;
mod ray;
mod sphere;
mod tri;
// mod trimesh;
mod utility;
mod vec3;
mod vec2;
mod aabb;
mod bvh;
mod color;
mod buffers;
mod render;
mod texture;
extern crate num_cpus;
use crate::render::{render_pixel, ray_color};
use crate::bvh::Bvh;
use crate::camera::Camera;
use crate::hit::{HitRecord, HittableList, Object, Hittable};
use crate::material::{Emits, Light, Material, Principle, Scatterable};
use crate::ray::Ray;
use crate::sphere::Sphere;
use crate::tri::Tri;
use crate::utility::{random_float, random_range, INF};
use crate::vec3::Vec3;
use crate::vec2::Vec2;
use crate::color::Color;
use crate::aabb::Aabb;
use crate::buffers::{Lobes, FrameBuffers};
use image::{DynamicImage, ImageBuffer, Rgb, Rgba, RgbImage, Rgb32FImage, Rgba32FImage};
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::{Result, Value};
use show_image::{create_window, ImageInfo, ImageView, WindowOptions};
use std::io::Write;
use std::collections::HashMap;
use std::time::Duration;
use std::{env, fs, thread};
use std::sync::{Arc, Mutex, RwLock};
use std::mem::drop;
use crate::texture::TextureMap;


#[show_image::main]
fn main() {

    print!("Processing scene...");
    let data = fs::read_to_string("render_data.json").expect("Unable to read render data.");
    // let data = fs::read_to_string("./src/scenes/roughness_wedge.json").expect("Unable to read render data."); 
    let data: serde_json::Value = serde_json::from_str(&data).expect("Incorrect JSON format.");

    // render settings
    let progressive = data["settings"]["progressive"].as_u64().unwrap() == 1;
    let aspect_ratio = data["settings"]["aspect_ratio"].as_f64().unwrap();
    let width = data["settings"]["width"].as_u64().unwrap() as u32;
    let height = (width as f64 / aspect_ratio) as u32;
    let fov = data["settings"]["fov"].as_f64().unwrap();
    let aperature = data["settings"]["aperature"].as_f64().unwrap();
    let cam_location: Vec3 = Vec3::new(
        data["settings"]["camera_origin"][0].as_f64().unwrap(),
        data["settings"]["camera_origin"][1].as_f64().unwrap(),
        data["settings"]["camera_origin"][2].as_f64().unwrap(),
    );
    let cam_aim: Vec3 = Vec3::new(
        data["settings"]["camera_aim"][0].as_f64().unwrap(),
        data["settings"]["camera_aim"][1].as_f64().unwrap(),
        data["settings"]["camera_aim"][2].as_f64().unwrap(),
    );
    let cam_focus: Vec3 = Vec3::new(
        data["settings"]["camera_focus"][0].as_f64().unwrap(),
        data["settings"]["camera_focus"][1].as_f64().unwrap(),
        data["settings"]["camera_focus"][2].as_f64().unwrap(),
    );

    let camera = Arc::new(Camera::new(
        fov,
        aspect_ratio,
        aperature,
        cam_location,
        cam_aim,
        cam_focus,
        0.0,
        1.0,
    ));
    let spp: u16 = data["settings"]["spp"].as_u64().unwrap() as u16;
    let depth: u32 = data["settings"]["depth"].as_u64().unwrap() as u32;
    let preview: RgbImage = ImageBuffer::new(width, height);
    let buffer_rgba: Arc<RwLock<Rgba32FImage>> = Arc::new(RwLock::new(ImageBuffer::new(width, height)));
    let buffer_diffuse: Arc<RwLock<Rgba32FImage>> = Arc::new(RwLock::new(ImageBuffer::new(width, height)));
    let buffer_specular: Arc<RwLock<Rgba32FImage>> = Arc::new(RwLock::new(ImageBuffer::new(width, height)));
    let render_view = ImageView::new(ImageInfo::rgb8(width, height), &preview);
    let window = create_window(
        "Krrust",
        WindowOptions::new()
            .set_size([width, height])
            .set_preserve_aspect_ratio(true)
            .set_borderless(false)
            .set_show_overlays(true),
    );
    window
        .as_ref()
        .expect("REASON")
        .set_image("image-001", render_view);
    let preview_output = "g:/krrusty_output_new.png";
    let mut output = data["settings"]["output_file"].to_string();
    output.pop();
    output.remove(0);
    let preview = Arc::new(RwLock::new(preview));

    // init world
    let mut world = HittableList::new();
    std::io::stdout().flush();
    println!("\rProcessing materials...");
    let mut scene_materials: HashMap<String, Arc<Material>> = HashMap::new();
    let material_array = &data["scene"]["materials"].as_array().unwrap();
    for mat in material_array.iter() {
        let name = mat["name"].to_string().replace(['"'], "");
        let diffuse = Color::new(
            mat["diffuse"][0].as_f64().unwrap(),
            mat["diffuse"][1].as_f64().unwrap(),
            mat["diffuse"][2].as_f64().unwrap(),
            1.0
        );
        let spec = mat["specular"][0].as_f64().unwrap();
        let ior = mat["ior"].as_f64().unwrap();
        let roughness = mat["roughness"][0].as_f64().unwrap();
        let diffuse_weight = mat["diffuse_weight"].as_f64().unwrap();
        let metallic = mat["metallic"].as_f64().unwrap();
        let refraction = mat["refraction"].as_f64().unwrap();
        let emissive = Color::new(
            mat["emissive"][0].as_f64().unwrap(),
            mat["emissive"][1].as_f64().unwrap(),
            mat["emissive"][2].as_f64().unwrap(),
            1.0
        );
        let material = Material::Principle(Principle::new(
            diffuse,
            spec,
            ior,
            roughness,
            diffuse_weight,
            metallic,
            refraction,
            emissive,
            None,
            None
        ));
        scene_materials.insert(name, Arc::new(material));
    }

    let skydome_texture = Arc::new(TextureMap::new("g:/rust_projects/krrust/textures/texture_sky_sunset.exr", false));
    let default_material =  Arc::new(Material::Principle(Principle::texture_test()));
    println!("Processing meshes...");
    // get tris
    let mesh_count = data["scene"]["mesh_count"].as_u64().unwrap();
    for obj in 0..mesh_count {
        let vtx_array = &data["scene"]["meshes"][obj as usize]["vertices"]
            .as_array()
            .unwrap();
        let normal_array = &data["scene"]["meshes"][obj as usize]["normals"]
            .as_array()
            .unwrap();
        let uv_array = &data["scene"]["meshes"][obj as usize]["uvs"]
        .as_array()
        .unwrap();
        for i in 0..vtx_array.len() {
            let p0 = Vec3::new(
                vtx_array[i][0][0].as_f64().unwrap(),
                vtx_array[i][0][1].as_f64().unwrap(),
                vtx_array[i][0][2].as_f64().unwrap(),
            );
            let p1 = Vec3::new(
                vtx_array[i][1][0].as_f64().unwrap(),
                vtx_array[i][1][1].as_f64().unwrap(),
                vtx_array[i][1][2].as_f64().unwrap(),
            );
            let p2 = Vec3::new(
                vtx_array[i][2][0].as_f64().unwrap(),
                vtx_array[i][2][1].as_f64().unwrap(),
                vtx_array[i][2][2].as_f64().unwrap(),
            );
            let n0 = Vec3::new(
                normal_array[i][0][0].as_f64().unwrap(),
                normal_array[i][0][1].as_f64().unwrap(),
                normal_array[i][0][2].as_f64().unwrap(),
            );
            let n1 = Vec3::new(
                normal_array[i][1][0].as_f64().unwrap(),
                normal_array[i][1][1].as_f64().unwrap(),
                normal_array[i][1][2].as_f64().unwrap(),
            );
            let n2 = Vec3::new(
                normal_array[i][2][0].as_f64().unwrap(),
                normal_array[i][2][1].as_f64().unwrap(),
                normal_array[i][2][2].as_f64().unwrap(),
            );
            let uv0 = Vec2::new(
                uv_array[i][0][0].as_f64().unwrap() as f32,
                uv_array[i][0][1].as_f64().unwrap() as f32
            );
            let uv1 = Vec2::new(
                uv_array[i][1][0].as_f64().unwrap() as f32,
                uv_array[i][1][1].as_f64().unwrap() as f32
            );
            let uv2 = Vec2::new(
                uv_array[i][2][0].as_f64().unwrap() as f32,
                uv_array[i][2][1].as_f64().unwrap() as f32
            );
            let vertices = vec![p0, p1, p2];
            let normals = vec![n0, n1, n2];
            let uvs = vec![uv0, uv1, uv2];
            let material_name = &data["scene"]["meshes"][obj as usize]["material"]
                .to_string()
                .replace(['"'], "");
            let material = scene_materials.get(material_name).unwrap();
            let new_tri = Object::Tri(Tri::new(vertices, normals, uvs, default_material.clone(), true));
            world.objects.push(Arc::new(new_tri));
            if vtx_array[i].as_array().unwrap().len() == 4 {
                let p3 = Vec3::new(
                    vtx_array[i][3][0].as_f64().unwrap(),
                    vtx_array[i][3][1].as_f64().unwrap(),
                    vtx_array[i][3][2].as_f64().unwrap(),
                );
                let n3 = Vec3::new(
                    normal_array[i][3][0].as_f64().unwrap(),
                    normal_array[i][3][1].as_f64().unwrap(),
                    normal_array[i][3][2].as_f64().unwrap(),
                ); 
                let uv3 = Vec2::new(
                    uv_array[i][3][0].as_f64().unwrap() as f32,
                    uv_array[i][3][1].as_f64().unwrap() as f32
                );
                let vertices = vec![p2, p3, p0];
                let normals = vec![n2, n3, n0];
                let uvs = vec![uv2, uv3, uv0];
                let quad_tri = Object::Tri(Tri::new(vertices, normals, uvs, default_material.clone(), true));
                world.objects.push(Arc::new(quad_tri));
            }
            
        }
    }

    // // get spheres
    let sphere_count = data["scene"]["sphere_count"].as_u64().unwrap();
    for obj in 0..sphere_count {
        let material_name = &data["scene"]["spheres"][obj as usize]["material"]
            .to_string()
            .replace(['"'], "");
        let x = data["scene"]["spheres"][obj as usize]["location"][0]
            .as_f64()
            .unwrap();
        let y = data["scene"]["spheres"][obj as usize]["location"][1]
            .as_f64()
            .unwrap();
        let z = data["scene"]["spheres"][obj as usize]["location"][2]
            .as_f64()
            .unwrap();
        let new_sphere = Object::Sphere(Sphere::new(
            Vec3::new(x, y, z),
            Vec3::new(x, y, z),
            0.0,
            1.0,
            data["scene"]["spheres"][obj as usize]["radius"]
                .as_f64()
                .unwrap(),
            scene_materials.get(material_name).unwrap().clone()
        ));
        world.objects.push(Arc::new(new_sphere));
    }

    // println!("Processing lights...");
    let light_count = data["scene"]["light_count"].as_u64().unwrap();
    for obj in 0..light_count {
        let vtx_array = &data["scene"]["lights"][obj as usize]["points"]
            .as_array()
            .unwrap();
        for i in 0..vtx_array.len() {
            let p0 = Vec3::new(
                vtx_array[i][0][0].as_f64().unwrap(),
                vtx_array[i][0][1].as_f64().unwrap(),
                vtx_array[i][0][2].as_f64().unwrap(),
            );
            let p1 = Vec3::new(
                vtx_array[i][1][0].as_f64().unwrap(),
                vtx_array[i][1][1].as_f64().unwrap(),
                vtx_array[i][1][2].as_f64().unwrap(),
            );
            let p2 = Vec3::new(
                vtx_array[i][2][0].as_f64().unwrap(),
                vtx_array[i][2][1].as_f64().unwrap(),
                vtx_array[i][2][2].as_f64().unwrap(),
            );
            let p3 = Vec3::new(
                vtx_array[i][3][0].as_f64().unwrap(),
                vtx_array[i][3][1].as_f64().unwrap(),
                vtx_array[i][3][2].as_f64().unwrap(),
            );

            let c = data["scene"]["lights"][obj as usize]["color"]
                .as_array()
                .unwrap();
            let r = c[0].as_f64().unwrap();
            let g = c[1].as_f64().unwrap();
            let b = c[2].as_f64().unwrap();
            let color = Color::new(r, g, b, 1.0);
            let intensity = data["scene"]["lights"][obj as usize]["intensity"]
                .as_f64()
                .unwrap();

            let material = Arc::new(Material::Light(Light::new(color, intensity)));
            let vertices = vec![p0, p1, p2];
            let vertices2 = vec![p2, p3, p0];
            let normals = vec![Vec3::black(), Vec3::black(), Vec3::black()];
            let normals2 = vec![Vec3::black(), Vec3::black(), Vec3::black()];
            let uvs = vec![Vec2::zero(), Vec2::zero(), Vec2::zero()];
            let uvs2 = vec![Vec2::zero(), Vec2::zero(), Vec2::zero()];
            let tri1 = Object::Tri(Tri::new(vertices, normals, uvs, material.clone(), false));
            let tri2 = Object::Tri(Tri::new(vertices2, normals2, uvs2, material.clone(), false));
            world.objects.push(Arc::new(tri1));
            world.objects.push(Arc::new(tri2));
        }
    }
    let world_size = &world.objects.len();

    println!("Processing BVH..."); 
    let world_bvh = Arc::new(Object::Bvh(Bvh::new(&mut world.objects, 0.0, 1.0)));


    println!("Rendering scene...");
    //----------------------------------------------------------------------------------
    //----------------------------------------------------------------------------------
    // PROGRESSIVE RENDERER 32-BIT
    //----------------------------------------------------------------------------------
    //----------------------------------------------------------------------------------
    let progress = ProgressBar::new((spp - 1) as u64).with_message("%...");
    progress.set_style(
        ProgressStyle::with_template("[{elapsed_precise}] {bar:40.gray} {percent}{msg}")
            .unwrap(),
    );

    // create pixel chunks for threads
    let thread_count = num_cpus::get() as u32;
    let width_vec: Arc<Vec<usize>> = Arc::new((0usize..width as usize).collect::<Vec<_>>());
    let mut chunks = Vec::new();
    let chunk_size = (width + thread_count - 1) / thread_count;
    for chunk in width_vec.chunks(chunk_size as usize) {
        chunks.push(chunk.to_owned());
    }
    let thread_chunks = Arc::new(chunks);

    for sample in 0..spp {
        if sample != 0 {
            progress.inc(1);
        } 
        for y in 0..height {
            // spawn threads and render chunks
            let mut threads:Vec<thread::JoinHandle<()>> = Vec::new();   
            for index in 2..thread_count+1 {
                let thread_chunks = thread_chunks.clone();
                let camera = camera.clone();
                let world_bvh = world_bvh.clone();
                let buffer_rgba = buffer_rgba.clone();
                let buffer_diff = buffer_diffuse.clone();
                let buffer_spec = buffer_specular.clone();
                let preview = preview.clone();
                let sky = skydome_texture.clone();
                threads.push(                    
                    thread::spawn( move || {          
                        for x in &thread_chunks[index as usize-1] {
                            render_pixel(
                                *x as u32, 
                                y, 
                                &height, 
                                &width, 
                                &sample, 
                                &buffer_rgba, 
                                &buffer_diff, 
                                &buffer_spec, 
                                &preview, 
                                &camera, 
                                &world_bvh, 
                                depth, 
                                depth,
                                progressive,
                                Some(sky.clone()),
                                false
                            );
                        }
                    })
                )
            }     
            // main thread - renders first horizontal chunk
            let buffer_rgba = buffer_rgba.clone();
            let buffer_diff = buffer_diffuse.clone();
            let buffer_spec = buffer_specular.clone();
            let preview = preview.clone();
            let camera = camera.clone();
            let world_bvh = world_bvh.clone();
            let thread_chunks = thread_chunks.clone();
            let sky = skydome_texture.clone();
            for x in &thread_chunks[0] {
                render_pixel(
                    *x as u32, 
                    y, 
                    &height, 
                    &width, 
                    &sample, 
                    &buffer_rgba, 
                    &buffer_diff, 
                    &buffer_spec, 
                    &preview, 
                    &camera, 
                    &world_bvh, 
                    depth, 
                    depth,
                    progressive,
                    Some(sky.clone()),
                    false
                );
            }
            for thread in threads {
                thread.join();
            }
        }
        let buffer = buffer_rgba.write().unwrap();
        buffer.save(&output);
        drop(buffer);
        let preview_buffer = preview.read().unwrap();
        let render_view = ImageView::new(ImageInfo::rgb8(width, height), &preview_buffer);
        window
            .as_ref()
            .expect("REASON")
            .set_image("image-001", render_view);
        drop(preview_buffer);
        let preview_buffer = preview.write().unwrap();
        preview_buffer.save(&preview_output);
        drop(preview_buffer);
    }
    let buffer = buffer_rgba.write().unwrap();
    buffer.save(&output);
    drop(buffer);
    ProgressBar::finish_with_message(&progress, "% Render complete")

}
