use crate::ray::Ray;
use crate::utility::{random_float, random_int, clamp, INF};
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
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord, lights: &Arc<Vec<Object>>) -> Option<(Ray, Color, Color, String)>;
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
    fn scatter(&self, ray: &Ray, hit_rec: &HitRecord, lights: &Arc<Vec<Object>>) -> Option<(Ray, Color, Color, String)> {
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
    pub bump: f64,
    pub bump_strength: f64,
    pub normal_strength: f64,
    pub diffuse_texture: Option<TextureMap>,
    pub diffuse_weight_texture: Option<TextureMap>,
    pub specular_texture: Option<TextureMap>,
    pub specular_weight_texture: Option<TextureMap>,
    pub roughness_texture: Option<TextureMap>,
    pub metallic_texture: Option<TextureMap>,
    pub refraction_texture: Option<TextureMap>,
    pub emission_texture: Option<TextureMap>,
    pub bump_texture: Option<TextureMap>,
    pub normal_texture: Option<TextureMap>,
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
        bump: f64,
        bump_strength: f64,
        normal_strength: f64,
        diffuse_texture: Option<TextureMap>,
        diffuse_weight_texture: Option<TextureMap>,
        specular_texture: Option<TextureMap>,
        specular_weight_texture: Option<TextureMap>,
        roughness_texture: Option<TextureMap>,
        metallic_texture: Option<TextureMap>,
        refraction_texture: Option<TextureMap>,
        emission_texture: Option<TextureMap>,
        bump_texture: Option<TextureMap>,
        normal_texture: Option<TextureMap>,
        
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
            bump,
            bump_strength,
            normal_strength,
            diffuse_texture,
            diffuse_weight_texture,
            specular_texture,
            specular_weight_texture,
            roughness_texture,
            metallic_texture,
            refraction_texture,
            emission_texture,
            bump_texture,
            normal_texture,
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
            bump: 0.0,
            bump_strength: 0.0,
            normal_strength: 0.0,
            diffuse_texture: None,
            diffuse_weight_texture: None,
            specular_texture: None,
            specular_weight_texture: None,
            roughness_texture: None,
            metallic_texture: None,
            refraction_texture: None,
            emission_texture: None,
            bump_texture: None,
            normal_texture: None
        }
    }
    
    pub fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
        r0 = r0 * r0;
        r0 + (1.0 - r0) * f64::powi(1.0 - cosine, 5)
    }

    pub fn scatter_pdf(r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        let cosine = Vec3::dot(&rec.normal, &scattered.direction.normalize());
        return if cosine < 0.0 {0.0} else {cosine / PI}
    }
}

impl Emits for Principle {
    fn emit(&self) -> Color {
        self.emission
    }
}

impl Scatterable for Principle {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, lights: &Arc<Vec<Object>>) -> Option<(Ray, Color, Color, String)> {
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

        let mut bump = 0.0;
        let mut bump_gradient = Color::black();
        let mut has_bump = false;
        if let Some(et) = &self.bump_texture {
            bump = self.bump_texture
                .as_ref()
                .map(|t| t.sample(rec.uv.x, rec.uv.y).r)
                .unwrap_or_else(|| 0.0);
            bump_gradient = self.bump_texture
                .as_ref()
                .map(|t| t.get_gradient(rec.uv.x, rec.uv.y))
                .unwrap_or_else(|| Color::black());
            has_bump = true;
        } 


        let mut has_normal = false;
        let mut normal_map = Color::black();
        if let Some(nm) = &self.normal_texture {
            normal_map = self.normal_texture
                .as_ref()
                .map(|t| t.sample(rec.uv.x, rec.uv.y))
                .unwrap_or_else(|| Color::black());
                has_normal = true;
        } 

        // bump map
        let mut perturbed_normal = rec.normal;
        let (t, b) = rec.normal.tangent_bitangent();
        if has_bump {            
            let bump_gradient = bump_gradient * self.bump_strength * 10.0;
            perturbed_normal = (
                rec.normal
                + (bump_gradient.g * t * rec.normal)
                + (bump_gradient.r * b * rec.normal))
                .normalize();     
        }

        //  normal map
        if has_normal {
            let normal_offset = normal_map.to_normal_vec(t, b, rec.normal) * self.normal_strength;
            perturbed_normal = (perturbed_normal + normal_offset).normalize();
        }

        // unit direction
        let unit_direction = r_in.direction.normalize();

        // light pdf
        let mut to_light = Vec3::zeros();
        let mut on_light = Vec3::zeros();
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

        // pick a light
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

        // generate ray direction based on chosen light
        if let Some(quad_light) = chosen_light {
            // pick a random point on the chosen light
            let (s, t) = (random_float(), random_float());
            on_light = quad_light.position
                + quad_light.x_axis * (s - 0.5) * quad_light.width
                + quad_light.y_axis * (t - 0.5) * quad_light.height;  

            // compute the direction
            to_light = (on_light - rec.point).normalize();
        }
        
        // compute probability of each lobe
        let roll = random_float();
        diffuse_weight = clamp(diffuse_weight - metallic - refraction, 0.0, 1.0);
        let metal = metallic > roll;
        let refract = refraction > roll * 2.0;
        let mut specular_prob = specular_weight / (specular_weight + diffuse_weight);

        // refraction
        if refract {               
            let cos_theta = f64::min(Vec3::dot(&(unit_direction * -1.0), &perturbed_normal), 1.0);
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

            let mut direction = unit_direction + Vec3::random_unit_vector() * roughness; 
            let real_refract = false;
            if real_refract {
                direction = Vec3::refract(&unit_direction, &rec.normal, refraction_ratio)
                     + Vec3::random_unit_vector() * roughness;
            }

            let scattered = Ray::new(rec.point, direction, r_in.time);
            let attenuation = Color::white();

            return Some((scattered, attenuation * 2.0, emission, "refraction".to_string()));
        } 
        
        // specular
        if specular_prob > roll {
            // reflectance values
            let ior_to_f0 = ((self.ior - 1.0) / (self.ior + 1.0)).powf(2.0);
            let basic_f0 = Vec3::new(ior_to_f0, ior_to_f0, ior_to_f0);
            let metal_f0 = Vec3::new(0.85, 0.85, 0.85);
            
            // roughness, view angle, normal
            let r = if roughness == 0.0 {0.001} else {roughness};
            let v = -r_in.direction.normalize();
            let mut n = perturbed_normal.normalize();
            let mut l = n;
            let mut h = n;
            
            // on axis check
            let t = 0.02;
            if n.x > 1.0-t && n.y < t && n.z < t
            || n.x < t && n.y > 1.0-t && n.z < t
            || n.x < t && n.y < t && n.z > 1.0-t 
            {n = n + Vec3::random_unit_vector() * roughness;}

            // determine direction of reflected ray
            let direct = random_float() < 0.5;
            if direct {
                // sample a light                          
                l = to_light;
                h = (v + l).normalize();
            } else {
                // sample random ggx vector
                h = ggx_sample(r, n).normalize();
                l = (2.0 * v.dot(&h) * h - v).normalize();
            }

            // scattered ray
            let scattered =  Ray::new(rec.point, l, r_in.time); 

            // dots
            let ndv = f64::max(n.dot(&v), 0.0);
            let ndh = f64::max(n.dot(&h), 0.0);
            let ndl = f64::max(n.dot(&l), 0.0);
            let ldh = f64::max(l.dot(&h), 0.0);

            // ggx
            let f0 = if metal {metal_f0} else {basic_f0};
            let d: f64 = ggx_distribution(ndh, r);
            let g: f64 = schlick_masking(ndl, ndv, r);
            let f: Color = schlick_fresnel(f0, ldh);
            let ggx =  f * g * d / f64::max((4.0 * ndv * ndl), 0.015);

            // compute weights
            let light_pdf = LightPdf::new(lights.clone(), rec.point, rec.normal);
            let direct_weight = light_pdf.value(&scattered.direction);
            let indirect_weight = d * ndh / (4.0 * ldh);
            let weight = direct_weight * 0.5 + indirect_weight * 0.5;                  

            // final color composite
            let attenuation = 
            if metal {diffuse * ggx * ndl / (weight * specular_prob)} 
            else {specular * ggx * ndl / (weight * specular_prob)};

            return Some((scattered, attenuation, emission, "specular".to_string()))          

        } else {            
            // diffuse
            let cosine_pdf = CosinePdf::new(perturbed_normal);
            let mut scattered = Ray::new(rec.point, cosine_pdf.generate(), r_in.time);

            // directly sample lights half the time
            let direct = random_float() > 0.5;
            if direct {    
                scattered.direction = to_light;               
            }

            // compute weights
            let light_pdf = LightPdf::new(lights.clone(), rec.point, perturbed_normal);
            let cosine_pdf_val = cosine_pdf.value(&scattered.direction) * 0.5;
            let light_pdf_val = light_pdf.value(&scattered.direction) * 0.5;
            let mut pdf = Principle::scatter_pdf(&r_in, &rec, &scattered);
            pdf = pdf / (cosine_pdf_val + light_pdf_val);

            // final color composite
            let attenuation = diffuse * diffuse_weight * pdf / (1.0 - specular_prob);

            return Some((scattered, attenuation, emission, "diffuse".to_string()))
        }      
        None     
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
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, lights: &Arc<Vec<Object>>) -> Option<(Ray, Color, Color, String)> {
        Some((*r_in, Color::black(), self.emit(), "emission".to_string()))
    }
}



fn reflect(i: &Vec3, n: &Vec3) -> Vec3 {
    return *i - *n * 2.0 *i.dot(n);
}


fn specular_pdf(cos_theta: f64, refraction_ratio: f64) -> f64 {
    let fresnel = Principle::reflectance(cos_theta, refraction_ratio);
    fresnel / std::f64::consts::PI
}

fn sample_lights(lights: &Arc<Vec<Object>>, point: Vec3) -> Vec3 {
    let mut to_light = Vec3::zeros();
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
    return to_light.normalize()
}

fn ggx_distribution(ndh: f64, roughness: f64) -> f64 {
    let a2: f64 = (roughness * roughness);
    let d: f64 = ((ndh * a2 - ndh) * ndh + 1.0);//ndh * ndh * (a2 -1.0 ) + 1.0;
    return a2 / (d * d * PI)
}

fn schlick_masking(ndl: f64, ndv: f64, roughness: f64) -> f64 {
    let k: f64 = roughness * roughness / 2.0;
    let g_v: f64 = ndv / (ndv * (1.0 - k) + k);
    let g_l: f64 = ndl / (ndl * (1.0 - k) + k);
    return g_v * g_l
}

fn schlick_masking_alt(ndl: f64, ndv: f64, roughness: f64) -> f64 {
    let a2: f64 = roughness * roughness;
    let g_v: f64 = ndl * (ndv * ndv * (1.0 - a2) + a2).sqrt();
    let g_l: f64 = ndv * (ndl * ndl * (1.0 - a2) + a2).sqrt();
    return 0.5 / (g_v + g_l)
}

fn schlick_fresnel(f0: Vec3, ldh: f64) -> Color {
    let f = f0 + (Vec3::ones() - f0) * (1.0 - ldh).powf(5.0);
    return Color::new(f.x, f.y, f.z, 1.0)
}

fn ggx_sample(roughness: f64, normal: Vec3) -> Vec3 {
    let (u, v) = (random_float(), random_float());
    let b: Vec3 = get_perpendicular(normal);
    let t: Vec3 = Vec3::cross(&b, &normal);
    let a2 = roughness * roughness;
    let cos_theta: f64 = (f64::max(0.0, (1.0 - u) / ((a2 - 1.0) * u + 1.0))).sqrt();
    let sin_theta: f64 = (f64::max(0.0, 1.0 - cos_theta * cos_theta)).sqrt();
    let phi = v * PI * 2.0;
    let direction = (t * (sin_theta * phi.cos()) + b * (sin_theta * phi.sin()) + normal * cos_theta);

    direction
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
    let tmp = Vec3::new(tmp[0], tmp[1], tmp[2]).normalize();
    let dot_product = perp.x * tmp.x + perp.y * tmp.y + perp.z * tmp.z;

    perp.x = perp.x - dot_product * tmp.x;
    perp.y = perp.y - dot_product * tmp.y;
    perp.z = perp.z - dot_product * tmp.z;

    perp.normalize()
}

