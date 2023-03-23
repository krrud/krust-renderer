use image::{DynamicImage, GenericImageView, Pixel};
use crate::color::Color;
use palette::{Srgb, LinSrgb};


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
        let y = ((1.0-v) * height as f32) as u32 % height;
        let pixel = self.image.get_pixel(x, y);
        let srgb = Srgb::new(
            f32::from(pixel[0]) / 255.0,
            f32::from(pixel[1]) / 255.0,
            f32::from(pixel[2]) / 255.0,
        ).into_linear();
        Color::new(srgb.red.into(), srgb.green.into(), srgb.blue.into(), 1.0)
    }
}
