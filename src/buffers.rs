use image::{DynamicImage, ImageBuffer, Rgb, Rgba, RgbImage, Rgb32FImage, Rgba32FImage};
use std::sync::{Arc, Mutex, RwLock};
use crate::color::Color;
use std::ops;


pub struct FrameBuffers {
    pub rgba: Rgba32FImage,
    pub diffuse: Rgba32FImage,
    pub specular: Rgba32FImage,
}

impl FrameBuffers {
    pub fn new(rgba: Rgba32FImage, diffuse: Rgba32FImage, specular: Rgba32FImage) -> Self {
        Self {
            rgba,
            diffuse,
            specular
        }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Lobes {
        let rgba = self.rgba.get_pixel(x, y);
        let rgba = Color::new(
            rgba[0] as f64, 
            rgba[1] as f64, 
            rgba[2] as f64, 
            rgba[3] as f64
        );
        let diffuse = self.diffuse.get_pixel(x, y);
        let diffuse = Color::new(
            diffuse[0] as f64, 
            diffuse[1] as f64, 
            diffuse[2] as f64, 
            diffuse[3] as f64
        );
        let specular = self.specular.get_pixel(x, y);
        let specular = Color::new(
            specular[0] as f64, 
            specular[1] as f64, 
            specular[2] as f64, 
            specular[3] as f64
        );
        Lobes::new(
            rgba,
            diffuse,
            specular,
            Color::black(),
            Color::black(),
            Color::black(),
            )
    }

    pub fn put_pixel(&mut self, x: u32, y: u32, rgba: Color, diffuse: Color, specular: Color) -> () {
        self.rgba.put_pixel(x, y, 
            Rgba([
                rgba.r as f32, 
                rgba.g as f32, 
                rgba.b as f32, 
                rgba.a as f32
                ]));
        self.diffuse.put_pixel(x, y, 
            Rgba([
                diffuse.r as f32, 
                diffuse.g as f32, 
                diffuse.b as f32, 
                diffuse.a as f32
                ]));
        self.specular.put_pixel(x, y,             
            Rgba([
                specular.r as f32, 
                specular.g as f32, 
                specular.b as f32, 
                specular.a as f32
            ]));
    }
}


#[derive(Debug, Clone, Copy)]
pub struct Lobes {
    pub rgba: Color,
    pub diffuse: Color,
    pub specular: Color, 
    pub emission: Color,
}

impl Lobes {
    pub fn new(rgba: Color, diffuse: Color, specular: Color, emission: Color) -> Self {
        Lobes {
            rgba,
            diffuse,
            specular,
            emission,
        }
    }

    pub fn empty() -> Self {
        Lobes {
            rgba: Color::black(),
            diffuse: Color::black(),
            specular: Color::black(),
            emission: Color::black(),
        }
    }

    pub fn average_samples(&self, sample: f64, average: f64, color: Lobes) -> Lobes {
        Lobes{
            rgba: (color.rgba + (self.rgba * sample)) / average,
            diffuse: (color.rgba + (self.rgba * sample)) / average,
            specular: (color.rgba + (self.rgba * sample)) / average,
            emission: self.emission,
        }
    }
}

impl ops::Add for Lobes {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            rgba: self.rgba + other.rgba,
            diffuse: self.diffuse + other.diffuse,
            specular: self.specular + other.specular,
            emission: self.emission + other.emission,
        }
    }
}

