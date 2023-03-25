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
use std::collections::HashMap;
use std::{env, fs, thread};
use std::sync::{Arc, Mutex, RwLock};
use crate::texture::TextureMap;
use std::io::Write;

pub fn process_scene(scene_file: &str) -> 
    (Camera, u16, u32, u32, u32, bool, Arc<Object>, Arc<TextureMap>, str, str, 
    Arc<RwLock<Rgba32FImage>>, Arc<RwLock<Rgba32FImage>>, Arc<RwLock<Rgba32FImage>>, RgbImage) {
    print!("Processing scene...");
    let scene = if scene_file.is_empty() {
        "render_data.json"
    } else {
        scene_file
    };
    let data = fs::read_to_string(scene).expect("Unable to read render data.");
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

        // textures
        let dt = mat["diffuse_tex"].to_string().replace(['"'], "");
        let mut diffuse_tex = None;
        if dt != "" {            
            diffuse_tex = Some(TextureMap::new(&dt, true))
        };

        let dwt = mat["diffuse_weight_tex"].to_string().replace(['"'], "");
        let mut diffuse_weight_tex = None;
        if dwt != "" {
            diffuse_weight_tex = Some(TextureMap::new(&dwt, true))
        };

        let st = mat["specular_tex"].to_string().replace(['"'], "");
        let mut specular_tex = None;
        if st != "" {
            specular_tex = Some(TextureMap::new(&st, true))
        };

        let swt = mat["specular_weight_tex"].to_string().replace(['"'], "");
        let mut specular_weight_tex = None;
        if swt != "" {
            specular_weight_tex = Some(TextureMap::new(&swt, true))
        };

        let rt = mat["roughness_tex"].to_string().replace(['"'], "");
        let mut roughness_tex = None;
        if rt != "" {
            roughness_tex = Some(TextureMap::new(&rt, true))
        };

        let mt = mat["metallic_tex"].to_string().replace(['"'], "");
        let mut metallic_tex = None;
        if mt != "" {
            metallic_tex = Some(TextureMap::new(&mt, true))
        };

        let rft = mat["refraction_tex"].to_string().replace(['"'], "");
        let mut refraction_tex = None;
        if rft != "" {
            refraction_tex = Some(TextureMap::new(&rft, true))
        };

        let et = mat["emission_tex"].to_string().replace(['"'], "");
        let mut emission_tex = None;
        if et != "" {
            emission_tex = Some(TextureMap::new(&et, true))
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
            diffuse_tex,
            diffuse_weight_tex,
            specular_tex,
            specular_weight_tex,
            roughness_tex,
            metallic_tex,
            refraction_tex,
            emission_tex
        ));
        scene_materials.insert(name, Arc::new(material));
    }

    let skydome_texture = Arc::new(TextureMap::new("g:/rust_projects/krrust/textures/texture_sky_sunset.exr", false));

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

    (camera, spp, width, height, depth, progressive, world_bvh, skydome_texture, output, preview_output, 
    buffer_rgba, buffer_diffuse, buffer_specular, preview)
}