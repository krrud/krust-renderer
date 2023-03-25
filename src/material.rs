use crate::hit::HitRecord;
use crate::ray::Ray;
use crate::utility::{random_float, random_range};
use crate::vec3::Vec3;
use crate::color::Color;
use crate::texture::TextureMap;


pub trait Scatterable {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<(Option<Ray>, Color, String, Color)>;
}

pub trait Emits {
    fn emit(&self) -> Color;
}

#[derive(Debug, Clone)]
pub enum Material {
    Principle(Principle),
    Light(Light),
}

impl Scatterable for Material {
    fn scatter(&self, ray: &Ray, hit_rec: &HitRecord) -> Option<(Option<Ray>, Color, String, Color)> {
        match self {
            Material::Principle(principle) => principle.scatter(ray, hit_rec),
            Material::Light(light) => light.scatter(ray, hit_rec),
            // Material::Metallic(metallic) => metallic.scatter(ray, hit_rec),
        }
    }
}
impl Emits for Material {
    fn emit(&self) -> Color {
        match self {
            Material::Principle(principle) => principle.emit(),
            Material::Light(light) => light.emit(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Principle {
    pub albedo: Color,
    pub spec: f64,
    pub ior: f64,
    pub roughness: f64,
    pub diffuse: f64,
    pub metallic: f64,
    pub refraction: f64,
    pub emissive: Color,
    pub diffuse_texture: Option<TextureMap>,
    pub roughness_texture: Option<TextureMap>
}

impl Principle {
    pub fn new(
        albedo: Color,
        spec: f64,
        ior: f64,
        roughness: f64,
        diffuse: f64,
        metallic: f64,
        refraction: f64,
        emissive: Color,
        diffuse_texture: Option<TextureMap>,
        roughness_texture: Option<TextureMap>
        
    ) -> Principle {
        Principle {
            albedo,
            spec,
            ior,
            roughness,
            diffuse,
            metallic,
            refraction,
            emissive,
            diffuse_texture,
            roughness_texture
        }
    }

    pub fn default() -> Self {
        Self {
            albedo: Color::black(),
            spec: 1.0,
            ior: 1.5,
            roughness: 0.5,
            diffuse: 1.0,
            metallic: 0.0,
            refraction: 0.0,
            emissive: Color::black(),
            diffuse_texture: None,
            roughness_texture: None,
        }
    }

    pub fn texture_test() -> Self {
        Self {
            albedo: Color::black(),
            spec: 1.0,
            ior: 1.5,
            roughness: 0.3,
            diffuse: 1.0,
            metallic: 0.0,
            refraction: 0.0,
            emissive: Color::black(),
            diffuse_texture: Some(TextureMap::new("g:/rust_projects/krrust/textures/crab/crab_color.tga", true)),//Some(TextureMap::new("g:/rust_projects/krrust/textures/crab/crab_color.tga", true)),
            roughness_texture: Some(TextureMap::new("g:/rust_projects/krrust/textures/crab/crab_roughness.png", true)),
        }
    }
    
    pub fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
        r0 = r0 * r0;
        r0 + (1.0 - r0) * f64::powi(1.0 - cosine, 5)
    }
}

impl Emits for Principle {
    fn emit(&self) -> Color {
        self.emissive
    }
}

impl Scatterable for Principle {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Option<Ray>, Color, String, Color)> {
        let mut roughness = self.roughness;
        if let Some(roughness_texture) = &self.roughness_texture {
            roughness = self.roughness_texture
                .as_ref()
                .map(|t| t.sample(rec.uv.x, rec.uv.y))
                .unwrap_or_else(|| Color::new(0.0, 1.0, 1.0, 1.0)).r;
        } 
        let roll = random_float();
        let unit_direction = Vec3::unit_vector(&r_in.direction);
        let cos_theta = f64::min(Vec3::dot(&(unit_direction * -1.0), &rec.normal), 1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let refraction_ratio = if cos_theta > 0.0 {
            if rec.front_face {
                1.0 / self.ior
            } else {
                self.ior
            }
        } else {
            self.ior
        };
        if 1.0 - self.metallic < roll {
            let reflected = Vec3::reflect(unit_direction, rec.normal);
            let scattered = Ray::new(
                rec.point,
                reflected + Vec3::random_unit_vector() * self.roughness,
                r_in.time,
            );
            let attenuation = self.albedo;
            if Vec3::dot(&scattered.direction, &rec.normal) > 0.0 {
                return Some((Some(scattered), attenuation, "specular".to_string(), self.emissive));
            } else {
                return None;
            }
        }
        if 1.0 - self.refraction < roll {
            let cannot_refract: bool = sin_theta * refraction_ratio > 1.0;
            let mut direction: Vec3;
            if cannot_refract
                || Principle::reflectance(cos_theta, refraction_ratio) > random_float()
            {
                direction = Vec3::reflect(unit_direction, rec.normal);
            } else {
                direction = Vec3::refract(&unit_direction, &rec.normal, refraction_ratio)
                    + Vec3::random_unit_vector() * self.roughness;
            }
            let scattered = Ray::new(rec.point, direction, r_in.time);
            let attenuation = Color::new(1.0, 1.0, 1.0, 1.0);
            return Some((Some(scattered), attenuation, "refraction".to_string(), self.emissive));
        }
        if 1.0 - self.diffuse < roll {
            let mut lobe = "diffuse";
            let mut attenuation = self.albedo;
            if let Some(diffuse_texture) = &self.diffuse_texture {
                attenuation = self.diffuse_texture
                    .as_ref()
                    .map(|t| t.sample(rec.uv.x, rec.uv.y))
                    .unwrap_or_else(|| Color::new(0.0, 1.0, 1.0, 1.0));
            } 
            let mut direction: Vec3 = rec.normal + Vec3::random_unit_vector();
            let spec_mult = random_float() <= self.spec;
            if Principle::reflectance(cos_theta, refraction_ratio)> random_float() && spec_mult {
                lobe = "specular";
                direction = Vec3::reflect(unit_direction, rec.normal)
                    + Vec3::random_unit_vector() * roughness;
                attenuation = Color::new(1.0, 1.0, 1.0, 1.0);
            }
            let scattered = Ray::new(rec.point, direction, r_in.time);
            Some((Some(scattered), attenuation, lobe.to_string(), self.emissive))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Light {
    pub color: Color,
    pub intensity: f64,
}

impl Light {
    pub fn new(color: Color, intensity: f64) -> Light {
        Light { color, intensity }
    }
}

impl Emits for Light {
    fn emit(&self) -> Color  {
        self.color * f64::powf(self.intensity, 2.0)
    }
}

impl Scatterable for Light {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Option<Ray>, Color, String, Color)> {
        Some((Some(*r_in), Color::black(), "emission".to_string(), self.emit()))
    }
}
