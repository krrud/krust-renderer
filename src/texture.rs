use image::{DynamicImage, GenericImageView, Pixel, ImageBuffer};
use image::codecs::hdr::{HdrDecoder};
use crate::color::Color;
use palette::{Srgb, LinSrgb};


#[derive(Debug, Clone)]
pub struct TextureMap {
    image: DynamicImage,
    srgb: bool
}

impl TextureMap {
    pub fn new(file_path: &str, srgb: bool) -> Self {    
        if srgb {
            let image =image::open(file_path).unwrap();
            TextureMap{ image, srgb}
        } else{
            let file = std::fs::File::open(file_path).unwrap();
            let reader = std::io::BufReader::new(file);
            let hdr_image = HdrDecoder::new(reader).unwrap();
            let metadata = hdr_image.metadata();
            let pixels = hdr_image.read_image_hdr().unwrap();
            let mut buffer_data = Vec::new();
            for pixel in pixels {
                buffer_data.push(pixel[0]*0.01);
                buffer_data.push(pixel[1]*0.01);
                buffer_data.push(pixel[2]*0.01);
            }
            let buffer = ImageBuffer::from_vec(metadata.width, metadata.height, buffer_data).unwrap();
            let image = DynamicImage::ImageRgb32F(buffer);

            TextureMap { image, srgb }
        }

    }
    
    pub fn sample(&self, u: f32, v: f32) -> Color {
        let (width, height) = self.image.dimensions();
        let x = (u * width as f32) as u32 % width;
        let y = ((1.0-v) * height as f32) as u32 % height;
        let pixel = self.image.get_pixel(x, y);
        if self.srgb {
            let srgb = Srgb::new(
            f32::from(pixel[0]) / 255.0,
            f32::from(pixel[1]) / 255.0,
            f32::from(pixel[2]) / 255.0,
            ).into_linear();
            Color::new(srgb.red.into(), srgb.green.into(), srgb.blue.into(), 1.0)
        } else {
            Color::new(pixel[0].into(), pixel[1].into(), pixel[2].into(), 1.0)
        }

    }
}
