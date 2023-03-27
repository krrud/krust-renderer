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
    pub diffuse: Color,
    pub diffuse_weight: f64,
    pub specular: Color,
    pub specular_weight: f64,
    pub roughness: f64,
    pub ior: f64,
    pub metallic: f64,
    pub refraction: f64,
    pub emission: Color,
    pub diffuse_texture: Option<TextureMap>,
    pub diffuse_weight_texture: Option<TextureMap>,
    pub specular_texture: Option<TextureMap>,
    pub specular_weight_texture: Option<TextureMap>,
    pub roughness_texture: Option<TextureMap>,
    pub metallic_texture: Option<TextureMap>,
    pub refraction_texture: Option<TextureMap>,
    pub emission_texture: Option<TextureMap>,
}

impl Principle {
    pub fn new(
        diffuse: Color,
        diffuse_weight: f64,
        specular: Color,
        specular_weight: f64,
        roughness: f64,
        ior: f64,
        metallic: f64,
        refraction: f64,
        emission: Color,
        diffuse_texture: Option<TextureMap>,
        diffuse_weight_texture: Option<TextureMap>,
        specular_texture: Option<TextureMap>,
        specular_weight_texture: Option<TextureMap>,
        roughness_texture: Option<TextureMap>,
        metallic_texture: Option<TextureMap>,
        refraction_texture: Option<TextureMap>,
        emission_texture: Option<TextureMap>,
        
    ) -> Principle {
        Principle {
            diffuse,
            diffuse_weight,
            specular,
            specular_weight,
            roughness,
            ior,
            metallic,
            refraction,
            emission,
            diffuse_texture,
            diffuse_weight_texture,
            specular_texture,
            specular_weight_texture,
            roughness_texture,
            metallic_texture,
            refraction_texture,
            emission_texture
        }
    }

    pub fn default() -> Self {
        Self {
            diffuse: Color::black(),
            diffuse_weight: 1.0,
            specular: Color::white(),
            specular_weight: 1.0,
            roughness: 0.5,
            ior: 1.5,
            metallic: 0.0,
            refraction: 0.0,
            emission: Color::black(),
            diffuse_texture: None,
            diffuse_weight_texture: None,
            specular_texture: None,
            specular_weight_texture: None,
            roughness_texture: None,
            metallic_texture: None,
            refraction_texture: None,
            emission_texture: None
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
        self.emission
    }
}

impl Scatterable for Principle {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Option<Ray>, Color, String, Color)> {
        // sample textures if available
        let mut diffuse = self.diffuse;
        if let Some(d) = &self.diffuse_texture {
            diffuse = self.diffuse_texture
                .as_ref()
                .map(|t| t.sample(rec.uv.x, rec.uv.y))
                .unwrap_or_else(|| Color::new(0.0, 1.0, 1.0, 1.0));
        } 

        let mut diffuse_weight = self.diffuse_weight;
        if let Some(dwt) = &self.diffuse_weight_texture {
            diffuse_weight = self.diffuse_weight_texture
                .as_ref()
                .map(|t| t.sample(rec.uv.x, rec.uv.y))
                .unwrap_or_else(|| Color::new(0.0, 1.0, 1.0, 1.0)).r;
        } 

        let mut specular = self.specular;
        if let Some(st) = &self.specular_texture {
            specular = self.specular_texture
                .as_ref()
                .map(|t| t.sample(rec.uv.x, rec.uv.y))
                .unwrap_or_else(|| Color::new(0.0, 1.0, 1.0, 1.0));
        } 

        let mut specular_weight = self.specular_weight;
        if let Some(swt) = &self.specular_weight_texture {
            specular_weight = self.specular_weight_texture
                .as_ref()
                .map(|t| t.sample(rec.uv.x, rec.uv.y))
                .unwrap_or_else(|| Color::new(0.0, 1.0, 1.0, 1.0)).r;
        }
        
        let mut roughness = self.roughness;
        if let Some(rt) = &self.roughness_texture {
            roughness = self.roughness_texture
                .as_ref()
                .map(|t| t.sample(rec.uv.x, rec.uv.y))
                .unwrap_or_else(|| Color::new(0.0, 1.0, 1.0, 1.0)).r;
        } 

        let mut metallic = self.metallic;
        if let Some(mt) = &self.metallic_texture {
            metallic = self.metallic_texture
                .as_ref()
                .map(|t| t.sample(rec.uv.x, rec.uv.y))
                .unwrap_or_else(|| Color::new(0.0, 1.0, 1.0, 1.0)).r;
        } 

        let mut refraction = self.refraction;
        if let Some(rft) = &self.refraction_texture {
            refraction = self.refraction_texture
                .as_ref()
                .map(|t| t.sample(rec.uv.x, rec.uv.y))
                .unwrap_or_else(|| Color::new(0.0, 1.0, 1.0, 1.0)).r;
        } 

        let mut emission = self.emission;
        if let Some(et) = &self.emission_texture {
            emission = self.emission_texture
                .as_ref()
                .map(|t| t.sample(rec.uv.x, rec.uv.y))
                .unwrap_or_else(|| Color::new(0.0, 1.0, 1.0, 1.0));
        } 

        // clip invalid weights
        diffuse_weight = f64::max(f64::min(diffuse_weight, 1.0), 0.0);
        specular_weight = f64::max(f64::min(specular_weight, 1.0), 0.0);

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
        // metallic
        if 1.0 - metallic < roll {
            let reflected = Vec3::reflect(unit_direction, rec.normal);
            let scattered = Ray::new(
                rec.point,
                reflected + Vec3::random_unit_vector() * roughness,
                r_in.time,
            );
            let attenuation = diffuse;
            if Vec3::dot(&scattered.direction, &rec.normal) > 0.0 {
                return Some((Some(scattered), attenuation, "specular".to_string(), emission));
            } else {
                return None;
            }
        }
        // refraction
        if 1.0 - refraction < roll {
            let cannot_refract: bool = sin_theta * refraction_ratio > 1.0;
            let mut direction: Vec3;
            if cannot_refract
                || Principle::reflectance(cos_theta, refraction_ratio) > random_float()
            {
                direction = Vec3::reflect(unit_direction, rec.normal);
            } else {
                direction = Vec3::refract(&unit_direction, &rec.normal, refraction_ratio)
                            + Vec3::random_unit_vector() * roughness;
            }
            let scattered = Ray::new(rec.point, direction, r_in.time);
            let attenuation = Color::new(1.0, 1.0, 1.0, 1.0);
            return Some((Some(scattered), attenuation, "refraction".to_string(), emission));
        } else {
            // diffuse and specular
            let mut lobe = "diffuse";
            let mut attenuation = diffuse * diffuse_weight; 
            let mut direction: Vec3 = rec.normal + Vec3::random_unit_vector();
            let spec_mult = random_float() <= specular_weight;
            if Principle::reflectance(cos_theta, refraction_ratio)> random_float() && spec_mult {
                lobe = "specular";
                direction = Vec3::reflect(unit_direction, rec.normal)
                            + Vec3::random_unit_vector() * roughness;
                attenuation = specular;
            }
            let scattered = Ray::new(rec.point, direction, r_in.time);
            Some((Some(scattered), attenuation, lobe.to_string(), emission))
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
