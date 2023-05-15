mod render_setup;
mod camera;
mod hit;
mod material;
mod ray;
mod sphere;
mod tri;
mod utility;
mod vec3;
mod vec2;
mod aabb;
mod bvh;
mod color;
mod buffers;
mod render;
mod texture;
mod lights;
mod onb;
mod pdf;
mod mat3;
use crate::render_setup::render_scene;


#[show_image::main]
fn main() {
    render_scene(
        Some("examples/dog.json"),
        "C:/krust_output/"
    );
 }

