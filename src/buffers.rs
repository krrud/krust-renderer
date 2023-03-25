use image::{DynamicImage, ImageBuffer, Rgb, Rgba, RgbImage, Rgb32FImage, Rgba32FImage};
use std::sync::{Arc, Mutex, RwLock};
use crate::color::Color;
use std::ops;

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

    pub fn empty() -> Self {
        Lobes {
            beauty: Color::new(0.0, 0.0, 0.0, 0.0),
            diffuse: Color::new(0.0, 0.0, 0.0, 0.0),
            specular: Color::new(0.0, 0.0, 0.0, 0.0),
            albedo: Color::new(0.0, 0.0, 0.0, 0.0),
            emission: Color::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn average_samples(&self, sample_count: u32) -> Lobes {
        Lobes{
            beauty: self.beauty / sample_count,
            diffuse: self.diffuse / sample_count,
            specular: self.specular / sample_count,
            albedo: self.albedo / sample_count,
            emission: self.emission / sample_count,
        }
    }
}

impl ops::Add for Lobes {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            beauty: self.beauty + other.beauty,
            diffuse: self.diffuse + other.diffuse,
            specular: self.specular + other.specular,
            albedo: self.albedo + other.albedo,
            emission: self.emission + other.emission,
        }
    }
}


pub struct FrameBuffers {
    diffuse: Arc<RwLock<Rgb32FImage>>,
    specular: Arc<RwLock<Rgb32FImage>>,
}

