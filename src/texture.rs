use image::{DynamicImage, GenericImageView, Pixel};
use crate::color::Color;


#[derive(Debug, Clone)]
pub struct TextureMap {
    image: DynamicImage,
}

impl TextureMap {
    pub fn new(file_path: &str) -> Self {
        let image = image::open(file_path).unwrap();
        TextureMap { image }
    }
    pub fn sample(&self, u: f32, v: f32) -> Color {
        let (width, height) = self.image.dimensions();
        let x = (u * width as f32) as u32 % width;
        let y = (v * height as f32) as u32 % height;
        let pixel = self.image.get_pixel(x, y);
        let (r, g, b) = (pixel[0], pixel[1], pixel[2]);
        Color::new(r.into(), g.into(), b.into(), 1.0)
    }
}
