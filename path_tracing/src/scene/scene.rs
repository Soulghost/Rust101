use std::{sync::Arc};

use crate::{math::{vector::Vector3f, Math}, mesh::{model::Model, object::Object}, bvh::bvh::BVH, domain::domain::{Ray, Intersection}};

pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub fov: f32,
    pub camera_background_color: Vector3f,
    pub russian_roulette: f32,
    pub sample_per_pixel: u32,
    models: Vec<Arc<Model>>,
    bvh: Option<BVH>
}

impl Scene {
    pub fn new(width: u32, 
               height: u32,
               fov: f32,
               camera_background_color: Vector3f,
               russian_roulette: f32,
               sample_per_pixel: u32) -> Scene {
        Scene { 
            width, 
            height, 
            fov, 
            camera_background_color, 
            russian_roulette,
            sample_per_pixel,
            models: vec![],
            bvh: None
        }
    }

    pub fn add(&mut self, model: Arc<Model>) {
        self.models.push(model);
    }

    pub fn build_bvh(&mut self) {
        println!("[Scene] Generating BVH...");
        let models = self.models.iter()
            .map(|model| {
                let obj: Arc<dyn Object> = model.clone();
                obj
            })
            .collect();
        let mut bvh = BVH::new(models);
        bvh.build();
        self.bvh = Some(bvh);
    }

    pub fn cast_ray(&self, ray: &Ray) -> Result<Vector3f, &'static str> {
        if self.bvh.is_none() {
            return Err("bvh not generated");
        }
        let inter = self.bvh.as_ref().unwrap().intersect(ray);
        if !inter.hit {
            return Ok(self.camera_background_color.clone());
        }
        let re_dir = -&ray.direction;
        return Ok(self.shade(&inter, &re_dir));
    }

    fn shade(&self, hit: &Intersection, wo: &Vector3f) -> Vector3f {
        if let Some(material) = &hit.material {
            if material.has_emission() {
                return material.get_emission();
            }
        }

        let (inter_light, pdf) = self.sample_light();
        let light_normal = &inter_light.normal;
        let ws = (&inter_light.coords - &hit.coords).normalize();
        let cosine_theta = ws.dot(&hit.normal);
        let cosine_theta_prime = (-&ws).dot(light_normal);

        // directional lighting
        let mut l_dir = Vector3f::zero();
        let hit_mat = hit.material.as_ref().unwrap();
        let hit_to_light_dis = inter_light.coords.distance_sq(&hit.coords) as f64;
        let shadow_check_inter = self.bvh.as_ref().unwrap().intersect(
            &Ray::new(&hit.coords, &ws, 0.0)
        );
        let occluder_dis = shadow_check_inter.distance * shadow_check_inter.distance;
        if occluder_dis - hit_to_light_dis > -f64::EPSILON {
            // not in shadow
            let f_r = hit_mat.eval(&ws, &wo, &hit.normal);
            l_dir = &hit.emit // L_i
                    * &f_r 
                    * cosine_theta
                    * cosine_theta_prime
                    / hit_to_light_dis
                    / pdf;
        }

        // indirectional lighting
        let mut l_indir = Vector3f::zero();
        if Math::sample_uniform_distribution(0.0, 1.0) < self.russian_roulette {
            let sample_dir = hit_mat.sample(&-wo, &hit.normal).normalize();
            
        }
        return l_dir + l_indir;
    }

    fn sample_light(&self) -> (Intersection, f32) {
        let mut emit_area_sum: f32 = 0.0;
        for obj in self.models.iter() {
            if obj.material.has_emission() {
                emit_area_sum += obj.get_area();
            }
        }

        let p = Math::sample_uniform_distribution(0.0, 1.0);
        emit_area_sum = 0.0;
        for obj in self.models.iter() {
            if obj.material.has_emission() {
                emit_area_sum += obj.get_area();
                if emit_area_sum >= p {
                    return obj.sample();
                }
            }
        }

        panic!("impossible");
    }
}