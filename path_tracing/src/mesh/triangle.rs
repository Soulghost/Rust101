use std::sync::Arc;

use crate::{material::material::Material, bvh::bounds::Bounds3, domain::domain::{Ray, Intersection}, math::vector::Vector3f};
use super::object::Object;

pub struct Triangle {
    pub v0: Vector3f,
    pub v1: Vector3f,
    pub v2: Vector3f,
    pub e1: Vector3f,
    pub e2: Vector3f,
    pub normal: Vector3f,
    pub area: f32,
    pub material: Arc<dyn Material>
}

impl Triangle {
    pub fn new(v0: &Vector3f, v1: &Vector3f, v2: &Vector3f, material: Arc<dyn Material>) -> Triangle {
        let e1 = v1 - v0;
        let e2 = v2 - v0; 
        Triangle { 
            v0: v0.clone(),
            v1: v1.clone(),
            v2: v2.clone(),
            normal: e1.cross(&e2).normalize(), 
            area: e1.cross(&e2).length(), 
            material,
            e1, e2,
        }
    }

    pub fn clone(&self) -> Triangle {
        Triangle { 
            v0: self.v0.clone(), 
            v1: self.v1.clone(),
            v2: self.v2.clone(), 
            e1: self.e1.clone(),
            e2: self.e2.clone(), 
            normal: self.normal.clone(), 
            area: self.area, 
            material: Arc::clone(&self.material) 
        }
    }
}

impl Object for Triangle {
    fn get_bounds(&self) -> Bounds3 {
        return Bounds3::zero();
    }

    fn get_area(&self) -> f32 {
        return 0.0;
    }

    fn intersect(&self, ray: &Ray) -> Intersection{
        !todo!()
    }
}