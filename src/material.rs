use crate::ray::Ray;
use crate::utility::{random_float, random_int};
use crate::vec3::Vec3;
use crate::color::Color;
use crate::texture::TextureMap;
use std::f64::consts::PI;
use crate::hit::{HitRecord, HittableList, Object, Hittable};
use std::sync::Arc;
use crate::onb::Onb;
use crate::pdf::{Pdf, CosinePdf, LightPdf};
use crate::lights::QuadLight;
use crate::vec2::Vec2;


pub trait Scatterable {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord, lights: &Arc<Vec<Object>>) -> Option<(Option<Ray>, Color, String, Color)>;
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
    fn scatter(&self, ray: &Ray, hit_rec: &HitRecord, lights: &Arc<Vec<Object>>) -> Option<(Option<Ray>, Color, String, Color)> {
        match self {
            Material::Principle(principle) => principle.scatter(ray, hit_rec, lights),
            Material::Light(light) => light.scatter(ray, hit_rec, lights),
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

    pub fn scatter_pdf(r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        let cosine = Vec3::dot(&rec.normal, &scattered.direction.unit_vector());
        return if cosine < 0.0 {0.0} else {cosine / PI}
    }
}

impl Emits for Principle {
    fn emit(&self) -> Color {
        self.emission
    }
}

impl Scatterable for Principle {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, lights: &Arc<Vec<Object>>) -> Option<(Option<Ray>, Color, String, Color)> {
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
            let offset = Vec3::random_unit_vector() * roughness;
            let offset = beckmann_offset(unit_direction, rec.normal, roughness);
            let scattered = Ray::new(
                rec.point,
                reflected + offset,
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
            // pdf
            let mut to_light = Vec3::black();
            let mut on_light = Vec3::black();
            let mut distance_squared = 0.0;
            let mut light_cosine = 0.0;
            let mut pdf_val = 0.0;
            let mut sum_pdf = 0.0;
            for (i, light) in lights.iter().enumerate() {
                match light {
                    Object::QuadLight(quad_light) => {
                        let distance_squared = (quad_light.position - rec.point).length_squared();
                        sum_pdf += quad_light.area / distance_squared;
                    }
                    _ => {}
                }
            }

            let mut chosen_light = None;
            for (i, light) in lights.iter().enumerate() {
                match light {
                    Object::QuadLight(quad_light) => {
                        let distance_squared = (quad_light.position - rec.point).length_squared();
                        let pdf = quad_light.area / distance_squared;
                        if chosen_light.is_none() && random_float() < pdf / sum_pdf {
                            chosen_light = Some(quad_light);
                        }
                    }
                    _ => {}
                }
            }

            // generate scatter direction based on pdf
            if let Some(quad_light) = chosen_light {
                // Generate a random point on the selected light
                let (s, t) = (random_float(), random_float());
                on_light = quad_light.position
                    + quad_light.x_axis * (s - 0.5) * quad_light.width
                    + quad_light.y_axis * (t - 0.5) * quad_light.height;  

                // Compute the direction to the random point on the light
                to_light = on_light - rec.point;
                distance_squared = to_light.length_squared();
                to_light = to_light.unit_vector();
                light_cosine = rec.normal.dot(&to_light).max(0.0);
                pdf_val = distance_squared / (light_cosine * quad_light.area);
            }
         
            if specular_weight / (specular_weight + diffuse_weight) > random_float() {
                // specular
                let r = if roughness == 0.0 {0.01} else {roughness*roughness};
                let v = -r_in.direction.unit_vector();
                let n = rec.normal.unit_vector();
                let mut h = n;
                let mut l = n;

                let direct = false;
                if direct {
                    // sample a light source
                    l = to_light.unit_vector();
                    h = (v + l).unit_vector();
                } else{
                    // random sample based on ggx distribution
                    h = ggx_sample(r, n);
                    l = (2.0 * v.dot(&h) * h - v);
                }          

                // dots
                let ndv = n.dot(&v);
                let ndh = n.dot(&h);
                let ldh = l.dot(&h);
                let ndl = n.dot(&l);

                // ggx 
                let d: f64 = ggx_distribution(ndh, r);
                let g: f64 = schlick_masking(ndl, ndv, r);
                let f: f64 = schlick_fresnel(0.05, ldh);
                let mut ggx = 0.0;
                if direct{
                    // ggx = d * g * f / (4.0 * ndl * ndv);
                    ggx = d * g * f / (4.0 * ndl / f64::max((ndv * ndh), 1e-5));
                } else {
                    ggx = d * g * f / (4.0 * ndl * ndv);
                }
                let direct_pdf = d * ndh / (4.0 * h.dot(&v));
                let indirect_pdf = d * ndh / (4.0 * ldh);
                let pdf = direct_pdf * 0.0 + indirect_pdf * 1.0;
                let attenuation = specular * ggx / pdf;
                let scattered =  Ray::new(rec.point, l, r_in.time); 

                // simple specular implementation
                // let offset = Vec3::random_unit_vector() * roughness;
                // let reflected_dir = Vec3::reflect(unit_direction, rec.normal) + offset; 
                // scattered =  Ray::new(rec.point, reflected_dir, r_in.time);  

                return Some((Some(scattered), attenuation, "specular".to_string(), emission))  

            } else {
                let mut attenuation = diffuse * diffuse_weight; 
                let cosine_pdf = CosinePdf::new(rec.normal);
                let light_pdf = LightPdf::new(lights.clone(), rec.point, rec.normal);
                let mut scattered = Ray::new(rec.point, cosine_pdf.generate(), r_in.time);
                if random_float() > 0.5 {          
                    scattered.direction = to_light;               
                }
                let cosine_pdf_val = cosine_pdf.value(&scattered.direction) * 0.5;
                let light_pdf_val = light_pdf.value(&scattered.direction) * 0.5;
                let mut pdf = Principle::scatter_pdf(&r_in, &rec, &scattered);
                pdf = pdf / (cosine_pdf_val + light_pdf_val);
                attenuation = attenuation * pdf;

                // simple diffuse implementation
                // let dir = rec.normal + Vec3::random_unit_vector();
                // scattered = Ray::new(rec.point, cosine_pdf.generate(), r_in.time);
                // attenuation = diffuse * diffuse_weight;

                return Some((Some(scattered), attenuation * 2.0, "diffuse".to_string(), emission)) 

            }      
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
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, lights: &Arc<Vec<Object>>) -> Option<(Option<Ray>, Color, String, Color)> {
        Some((Some(*r_in), Color::black(), "emission".to_string(), self.emit()))
    }
}



fn reflect(i: &Vec3, n: &Vec3) -> Vec3 {
    return *i - *n * 2.0 *i.dot(n);
}


fn beckmann_offset(direction: Vec3, normal: Vec3, roughness: f64) -> Vec3 {
    let e = 1e-6;
    let mut r = roughness;
    if roughness == 0.0 {
        r = e;
    } else if roughness == 1.0 {
        r = 1.0 - e;
    }
    let alpha = f64::sqrt(2.0) * r;
    let microfacet_normal = Vec3::random_unit_vector();
    let halfway = Vec3::reflect(direction, normal) + microfacet_normal * alpha;
    let cos_theta_h = Vec3::dot(&halfway, &normal);
    let beckmann_d = (f64::exp(-f64::tan(cos_theta_h).powf(2.0) / alpha.powf(2.0))) / (PI * alpha.powf(2.0) * cos_theta_h.powf(4.0));

    return microfacet_normal * beckmann_d;
}

fn specular_pdf(cos_theta: f64, refraction_ratio: f64) -> f64 {
    let fresnel = Principle::reflectance(cos_theta, refraction_ratio);
    fresnel / std::f64::consts::PI
}

fn importance_sample_lights(lights: Vec<Object>, point: Vec3) -> Vec3 {
    let mut to_light = Vec3::black();
    let mut sum_pdf = 0.0;
    for (i, light) in lights.iter().enumerate() {
        match light {
            Object::QuadLight(quad_light) => {
                let distance_squared = (quad_light.position - point).length_squared();
                sum_pdf += quad_light.area / distance_squared;
            }
            _ => {}
        }
    }

    let mut chosen_light = None;
    for (i, light) in lights.iter().enumerate() {
        match light {
            Object::QuadLight(quad_light) => {
                let distance_squared = (quad_light.position - point).length_squared();
                let pdf = quad_light.area / distance_squared;
                if chosen_light.is_none() && random_float() < pdf / sum_pdf {
                    chosen_light = Some(quad_light);
                }
            }
            _ => {}
        }
    }

    // generate scatter direction based on pdf
    if let Some(quad_light) = chosen_light {
        // Generate a random point on the selected light
        let (s, t) = (random_float(), random_float());
        let on_light = quad_light.position
            + quad_light.x_axis * (s - 0.5) * quad_light.width
            + quad_light.y_axis * (t - 0.5) * quad_light.height;  

        // Compute the direction to the random point on the light
        let to_light = on_light - point;    
    }   
    return to_light.unit_vector()
}

fn ggx_distribution(ndh: f64, roughness: f64) -> f64 {
    let a2: f64 = roughness * roughness;
    let d: f64 = ((ndh * a2 - ndh) * ndh + 1.0);
    return a2 / (d * d * PI)
}

fn schlick_masking(ndl: f64, ndv: f64, roughness: f64) -> f64 {
    let k: f64 = roughness * roughness / 2.0;
    let g_v: f64 = ndv / (ndv * (1.0 - k) + k);
    let g_l: f64 = ndl / (ndl * (1.0 - k) + k);
    return g_v * g_l
}

fn schlick_fresnel(f0: f64, ldh: f64) -> f64 {
    // let f0_vec = Vec3::new(f0, f0, f0);
    return f0 + (1.0 - f0) * (1.0 - ldh).powf(5.0)
}

fn ggx_sample(roughness: f64, normal: Vec3) -> Vec3 {
    let (u, v) = (random_float(), random_float());
    let b: Vec3 = get_perpendicular(normal).unit_vector();
    let t: Vec3 = Vec3::cross(&b, &normal).unit_vector();
    let a2 = roughness * roughness;
    let cos_theta: f64 = (f64::max(0.0, (1.0 - u) / ((a2 - 1.0) * u + 1.0))).sqrt();
    let sin_theta: f64 = (f64::max(0.0, 1.0 - cos_theta * cos_theta)).sqrt();
    let phi = v * PI * 2.0;
    let direction = t.unit_vector() * (sin_theta * phi.cos()) + b.unit_vector() * (sin_theta * phi.sin()) + normal * cos_theta;

    direction.unit_vector()
}


fn get_perpendicular(vec: Vec3) -> Vec3 {
    let mut smallest = 0;
    let mut perp = vec;
    let v = [vec.x, vec.y, vec.z];
    for i in 1..3 {
        if v[i].abs() < v[smallest].abs() {
            smallest = i;
        }
    }

    let mut tmp = [0.0; 3];
    tmp[smallest] = 1.0;
    let tmp = Vec3::new(tmp[0], tmp[1], tmp[2]).unit_vector();
    let dot_product = perp.x * tmp.x + perp.y * tmp.y + perp.z * tmp.z;

    perp.x = perp.x - dot_product * tmp.x;
    perp.y = perp.y - dot_product * tmp.y;
    perp.z = perp.z - dot_product * tmp.z;

    perp.unit_vector()
}