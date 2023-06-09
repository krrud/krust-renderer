extern crate num_cpus;
use crate::render::{ray_color, get_pixel_chunks, render_chunk};
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
use image::{DynamicImage, ImageBuffer, Rgb, Rgba, RgbImage, RgbaImage, Rgb32FImage, Rgba32FImage};
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
use rayon::prelude::*;
use crate::lights::{QuadLight, DirectionalLight};


pub fn render_scene(scene_file: Option<&str>, output_dir: &str) -> () {

    print!("Processing scene...");
    let mut data: serde_json::Value = serde_json::Value::Null;
    if let Some(file) = scene_file {
        let data_read = fs::read_to_string(file).expect("Unable to read render data.");
        data = serde_json::from_str(&data_read).expect("Incorrect JSON format.");
    } else {
        let data_read = fs::read_to_string("render_data.json").expect("Unable to read render data.");
        data = serde_json::from_str(&data_read).expect("Incorrect JSON format."); 
    }

    // extract render settings
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

    // create framebuffers and viewer
    let mut preview: RgbaImage = ImageBuffer::new(width, height);
    let mut preview_diff: RgbaImage = ImageBuffer::new(width, height);
    let buffer_rgba: Rgba32FImage = ImageBuffer::new(width, height);
    let buffer_diffuse: Rgba32FImage = ImageBuffer::new(width, height);
    let buffer_specular: Rgba32FImage = ImageBuffer::new(width, height);
    let mut buffers = FrameBuffers::new(buffer_rgba, buffer_diffuse, buffer_specular);
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
    let output_file = output_dir.to_owned() + "krust_render.png";

    // init world
    let mut world = HittableList::new();
    let mut quad_lights: Vec<Object> = vec![];
    let mut dir_lights: Vec<DirectionalLight> = vec![];

    // get materials
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
        let specular = Color::new(
            mat["specular"][0].as_f64().unwrap(),
            mat["specular"][1].as_f64().unwrap(),
            mat["specular"][2].as_f64().unwrap(),
            1.0
        );
        let specular_weight = mat["specular_weight"][0].as_f64().unwrap();
        let ior = mat["ior"].as_f64().unwrap();
        let roughness = mat["roughness"][0].as_f64().unwrap();
        let diffuse_weight = mat["diffuse_weight"][0].as_f64().unwrap();
        let metallic = mat["metallic"][0].as_f64().unwrap();
        let refraction = mat["refraction"][0].as_f64().unwrap();
        let emission = Color::new(
            mat["emission"][0].as_f64().unwrap(),
            mat["emission"][1].as_f64().unwrap(),
            mat["emission"][2].as_f64().unwrap(),
            1.0
        );
        let bump = mat["bump"][0].as_f64().unwrap();
        let bump_strength = mat["bump_strength"].as_f64().unwrap();
        let normal_strength = mat["normal_strength"].as_f64().unwrap();
        
        // textures
        let mut diffuse_tex = None;
        let dt = mat["diffuse_tex"].to_string().replace(['"'], "");
        if dt != "" {            
            diffuse_tex = Some(TextureMap::new(&dt, true))
        };

        let mut diffuse_weight_tex = None;
        let dwt = mat["diffuse_weight_tex"].to_string().replace(['"'], "");
        if dwt != "" {
            diffuse_weight_tex = Some(TextureMap::new(&dwt, true))
        };

        let mut specular_tex = None;
        let st = mat["specular_tex"].to_string().replace(['"'], "");
        if st != "" {
            specular_tex = Some(TextureMap::new(&st, true))
        };

        let mut specular_weight_tex = None;
        let swt = mat["specular_weight_tex"].to_string().replace(['"'], "");
        if swt != "" {
            specular_weight_tex = Some(TextureMap::new(&swt, true))
        };

        let mut roughness_tex = None;
        let rt = mat["roughness_tex"].to_string().replace(['"'], "");
        if rt != "" {
            roughness_tex = Some(TextureMap::new(&rt, true))
        };

        let mut metallic_tex = None;
        let mt = mat["metallic_tex"].to_string().replace(['"'], "");
        if mt != "" {
            metallic_tex = Some(TextureMap::new(&mt, true))
        };

        let mut refraction_tex = None;
        let rft = mat["refraction_tex"].to_string().replace(['"'], "");
        if rft != "" {
            refraction_tex = Some(TextureMap::new(&rft, true))
        };

        let mut emission_tex = None;
        let et = mat["emission_tex"].to_string().replace(['"'], "");
        if et != "" {
            emission_tex = Some(TextureMap::new(&et, true))
        };

        let mut bump_tex = None;
        let bt = mat["bump_tex"].to_string().replace(['"'], "");
        if bt != "" {
            bump_tex = Some(TextureMap::new(&bt, true))
        };

        let mut normal_tex = None;
        let nt = mat["normal_tex"].to_string().replace(['"'], "");
        if nt != "" {
            normal_tex = Some(TextureMap::new(&nt, true))
        };

        let material = Material::Principle(Principle::new(
            diffuse,
            diffuse_weight,
            specular,
            specular_weight,
            roughness,
            ior,
            metallic,
            refraction,
            emission,
            bump,
            bump_strength,
            normal_strength,
            diffuse_tex,
            diffuse_weight_tex,
            specular_tex,
            specular_weight_tex,
            roughness_tex,
            metallic_tex,
            refraction_tex,
            emission_tex,
            bump_tex,
            normal_tex
        ));
        scene_materials.insert(name, Arc::new(material));
    }

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
            let new_tri = Object::Tri(Tri::new(vertices, normals, uvs, material.clone(), true));
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
                let quad_tri = Object::Tri(Tri::new(vertices, normals, uvs, material.clone(), true));
                world.objects.push(Arc::new(quad_tri));
            }
            
        }
    }

    // get spheres
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

    // get quad lights
    let count = data["scene"]["quad_light_count"].as_u64().unwrap();
    for obj in 0..count {
        let vtx_array = &data["scene"]["lights"]["quad"][obj as usize]["points"]
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

            let c = data["scene"]["lights"]["quad"][obj as usize]["color"]
                .as_array()
                .unwrap();
            let r = c[0].as_f64().unwrap();
            let g = c[1].as_f64().unwrap();
            let b = c[2].as_f64().unwrap();
            let color = Color::new(r, g, b, 1.0);
            let intensity = data["scene"]["lights"]["quad"][obj as usize]["intensity"]
                .as_f64()
                .unwrap();
            let vertices = vec![p0, p1, p2, p3];
            let light = Object::QuadLight(QuadLight::new(color, intensity, vertices));            
            quad_lights.push(light);
            let vertices = vec![p0, p1, p2, p3];
            let light2 = Object::QuadLight(QuadLight::new(color, intensity, vertices));
            world.objects.push(Arc::new(light2));
        }
    }

    // get dir lights
    let count = data["scene"]["dir_light_count"].as_u64().unwrap();
    for obj in 0..count {
        let c = data["scene"]["lights"]["dir"][obj as usize]["color"]
            .as_array()
            .unwrap();
        let r = c[0].as_f64().unwrap();
        let g = c[1].as_f64().unwrap();
        let b = c[2].as_f64().unwrap();
        let color = Color::new(r, g, b, 1.0);
        let intensity = data["scene"]["lights"]["dir"][obj as usize]["intensity"]
            .as_f64()
            .unwrap();
        let softness = data["scene"]["lights"]["dir"][obj as usize]["softness"]
        .as_f64()
        .unwrap();
        let dir_array = data["scene"]["lights"]["dir"][obj as usize]["direction"]
            .as_array()
            .unwrap();
        let direction = Vec3::new(
            dir_array[0].as_f64().unwrap(), 
            dir_array[1].as_f64().unwrap(), 
            dir_array[2].as_f64().unwrap()
        );
        let light = DirectionalLight::new(direction, color, intensity, softness);            
        dir_lights.push(light);
    }

    let quad_lights = Arc::new(quad_lights);
    let dir_lights = Arc::new(dir_lights);

    println!("Processing BVH..."); 
    let world_bvh = Arc::new(Object::Bvh(Bvh::new(&mut world.objects, 0.0, 1.0)));

    //----------------------------------------------------------------------------------
    //----------------------------------------------------------------------------------
    // PROGRESSIVE RENDERER 32-BIT
    //----------------------------------------------------------------------------------
    //----------------------------------------------------------------------------------
    println!("Rendering scene...");

    let progress = ProgressBar::new((spp - 1) as u64).with_message("%...");
    progress.set_style(
        ProgressStyle::with_template("[{elapsed_precise}] {bar:40.gray} {percent}{msg}")
            .unwrap(),
    );

    let pixel_chunks = Arc::new(get_pixel_chunks(64 as usize, width as usize, height as usize)); 
    let num_threads = num_cpus::get();
    let thread_chunk_size = (pixel_chunks.len() as f32 / num_threads as f32).ceil() as usize;
    for sample in 0..spp {
        if sample != 0 {
            progress.inc(1);
        }
        let skydome_texture = Arc::new(TextureMap::new("g:/rust_projects/krrust/textures/alley_01.jpg", true));
        let mut handles = Vec::with_capacity(num_threads);
        for chunk in pixel_chunks.chunks(thread_chunk_size).map(|c| c.to_vec()) {
            let camera = camera.clone();
            let world_bvh = world_bvh.clone();
            let quad_lights = quad_lights.clone();
            let dir_lights = dir_lights.clone();
            let sky = skydome_texture.clone();
            let handle = thread::spawn(move || {
                let result = chunk.iter().map(|c|
                    render_chunk(
                        c,
                        height,
                        width,
                        &sample,
                        &camera,
                        &world_bvh,
                        &quad_lights,
                        &dir_lights,
                        depth,
                        depth,
                        progressive,
                        &None,//&Some(sky.clone()),
                        false,
                        )
                    ).collect::<Vec<Vec<(u32, u32, Lobes)>>>();
                result
            });
            handles.push(handle);
        }

        for handle in handles {
            let thread_result = handle.join().unwrap();
            for chunk_result in thread_result.iter() {
                for pixel in chunk_result{
                    let (x, y, color) = (pixel.0, pixel.1, pixel.2);
                    let (mut rgba, mut diff, mut spec) = (color.rgba, color.diffuse, color.specular);
                    let previous = buffers.get_pixel(x, y);
                    let (previous_rgba, previous_diff, previous_spec) = (
                        previous.rgba, 
                        previous.diffuse, 
                        previous.specular
                    );

                    // average in new sample
                    if sample > 0 {
                        let average = (sample + 1) as f64;
                        rgba = (rgba + (previous_rgba * sample as f64)) / average;
                        diff = (diff + (previous_diff * sample as f64)) / average;
                        spec = (spec + (previous_spec * sample as f64)) / average;
                    }

                    buffers.put_pixel(x, y, rgba, diff, spec);
                    preview.put_pixel(
                        x,
                        y,
                        Rgba([
                            (rgba.r.sqrt() * 255.999) as u8,
                            (rgba.g.sqrt() * 255.999) as u8,
                            (rgba.b.sqrt() * 255.999) as u8,
                            255 as u8,
                        ]),
                    );
                    preview_diff.put_pixel(
                        x,
                        y,
                        Rgba([
                            (diff.r.sqrt() * 255.999) as u8,
                            (diff.g.sqrt() * 255.999) as u8,
                            (diff.b.sqrt() * 255.999) as u8,
                            (diff.a.sqrt() * 255.999) as u8,
                        ]),
                    );
                }
            }
        }
        let render_view = ImageView::new(ImageInfo::rgba8(width, height), &preview);
        window
            .as_ref()
            .expect("REASON")
            .set_image("image-001", render_view);
        preview.save(&output_file);
    }
    // buffers.rgba.save(&output);
    ProgressBar::finish_with_message(&progress, "% Render complete")
}
