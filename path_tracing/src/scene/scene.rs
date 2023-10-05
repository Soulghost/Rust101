use std::sync::Arc;

use crate::{math::vector::Vector3f, mesh::{model::Model, object::Object}, bvh::bvh::BVH, domain::domain::{Ray, Intersection}};

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

    fn shade(&self, intersection: &Intersection, wo: &Vector3f) -> Vector3f {
        if let Some(material) = &intersection.material {
            if material.has_emission() {
                return material.get_emission();
            }
        }

        return self.camera_background_color.clone();
    }
}