use image::{DynamicImage, ImageBuffer, Rgb, Rgba, RgbImage, Rgb32FImage, Rgba32FImage};
use std::sync::{Arc, Mutex, RwLock};
use crate::color::Color;


pub struct Lobes {
    pub beauty: Color,
    pub diffuse: Color,
    pub specular: Color,
    pub albedo: Color, 
    pub emission: Color,
}

impl Lobes {
    pub fn new(self, beauty: Color, diffuse: Color, specular: Color, albedo: Color, emission: Color) -> Self {
        Lobes {
            beauty,
            diffuse,
            specular,
            albedo,
            emission,
        }
    }

    pub fn empty(self) -> Self {
        Lobes {
            beauty: Color::black(),
            diffuse: Color::black(),
            specular: Color::black(),
            albedo: Color::black(),
            emission: Color::black(),
        }
    }
}


pub struct FrameBuffers {
    diffuse: Arc<RwLock<Rgb32FImage>>,
    specular: Arc<RwLock<Rgb32FImage>>,
}

