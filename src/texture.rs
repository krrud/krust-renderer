use image::{DynamicImage, GenericImageView, Pixel, ImageBuffer, Rgb};
use image::codecs::hdr::{HdrDecoder, Rgbe8Pixel};
use crate::color::Color;
use palette::{Srgb, LinSrgb};
use std::path::Path;
use rayon::prelude::*;
use crate::vec3::Vec3;

#[derive(Debug, Clone)]
pub struct TextureMap {
    pub image: ImageBuffer<Rgb<f32>, Vec<f32>>,
    srgb: bool
}

impl TextureMap {
    pub fn new(file_path: &str, srgb: bool) -> Self {    
        if srgb {
            let image =image::open(file_path).unwrap().into_rgb32f();
            TextureMap{image, srgb}
        } else{
            let ext = Path::new(file_path).extension().unwrap();
            if ext == "hdr" {                
                let file = std::fs::File::open(file_path).unwrap();
                let reader = std::io::BufReader::new(file);
                let decoder = HdrDecoder::new(reader).unwrap();
                let metadata = decoder.metadata();
                let pixels = decoder.read_image_hdr().unwrap();
                let buffer_data = pixels.par_chunks(1000)
                    .flat_map(|chunk| {
                        let mut buffer_data = Vec::new();
                        for pixel in chunk {
                            buffer_data.push(pixel[0]);
                            buffer_data.push(pixel[1]);
                            buffer_data.push(pixel[2]);
                        }
                        buffer_data
                    })
                .collect::<Vec<_>>();
                let buffer = ImageBuffer::from_raw(metadata.width, metadata.height, buffer_data).unwrap();
                TextureMap {image: buffer, srgb}
            } else {
                let image =image::open(file_path).unwrap().into_rgb32f();
                TextureMap{image, srgb}
            }

        }
    }
    
    pub fn sample(&self, u: f32, v: f32) -> Color {
        let (width, height) = self.image.dimensions();
        let x = (u * width as f32) as u32 % width;
        let y = ((1.0-v) * height as f32) as u32 % height;
        let pixel = self.image.get_pixel(x, y);
        if self.srgb {
            let srgb = Srgb::new(
            f32::from(pixel[0]),
            f32::from(pixel[1]),
            f32::from(pixel[2]),
            ).into_linear();
            Color::new(srgb.red.into(), srgb.green.into(), srgb.blue.into(), 1.0)
        } else {
            Color::new(pixel[0].into(), pixel[1].into(), pixel[2].into(), 1.0)
        }

    }


    pub fn get_gradient(&self, u: f32, v: f32) -> Color {
        let (width, height) = self.image.dimensions();
        let x = (u * width as f32) as u32 % width;
        let y = ((1.0-v) * height as f32) as u32 % height;

        let x_next = if x + 1 > width-1 {x} else {x+1};
        let y_next = if y + 1 > height-1 {y} else {y+1};

        // Compute the gradient of the bump height using nearby points
        let dx = self.sample_pixel(x_next, y).r - self.sample_pixel(x, y).r;
        let dy = self.sample_pixel(x, y_next).r - self.sample_pixel(x, y).r;

        Color::new(dx, dy, 0.0, 1.0)
    }

    pub fn sample_pixel(&self, x: u32, y: u32) -> Color {
        let pixel = self.image.get_pixel(x, y);
        if self.srgb {
            let srgb = Srgb::new(
            f32::from(pixel[0]),
            f32::from(pixel[1]),
            f32::from(pixel[2]),
            ).into_linear();
            Color::new(srgb.red.into(), srgb.green.into(), srgb.blue.into(), 1.0)
        } else {
            Color::new(pixel[0].into(), pixel[1].into(), pixel[2].into(), 1.0)
        }
    }

}




