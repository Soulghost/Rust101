use std::f32::EPSILON;

use crate::math::vector::Vector3f;

pub trait Material : Send + Sync {
    fn get_albedo(&self) -> Vector3f;
    fn has_emission(&self) -> bool;
    fn get_emission(&self) -> Vector3f;
}

pub struct LitMaterial {
    pub emission: Vector3f,
    pub albedo: Vector3f,
}

impl LitMaterial {
    pub fn new(albedo: &Vector3f, emission: &Vector3f) -> LitMaterial {
        LitMaterial {
            albedo: albedo.clone(),
            emission: emission.clone()
        }
    }
}

impl Material for LitMaterial {
    fn get_albedo(&self) -> Vector3f {
        return self.albedo.clone();
    }

    fn has_emission(&self) -> bool {
        return self.emission.length() > EPSILON;
    }

    fn get_emission(&self) -> Vector3f {
        return self.emission.clone();
    }
}