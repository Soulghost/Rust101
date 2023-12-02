use std::{cell::RefCell, mem::transmute};

use cgmath::num_traits::ToPrimitive;

use crate::math::Vector3f;

pub mod pbr;

pub struct PBRMaterial {
    pub albedo: Vector3f,
    pub emission: Vector3f,
    pub metallic: f64,
    pub roughness: f64,
    pub ao: f64,
    pub index: RefCell<i32>,
}

impl PBRMaterial {
    pub fn new(
        albedo: Vector3f,
        emission: Vector3f,
        metallic: f64,
        roughness: f64,
        ao: f64,
    ) -> PBRMaterial {
        PBRMaterial {
            albedo,
            emission,
            metallic,
            roughness,
            ao,
            index: RefCell::new(-1),
        }
    }

    pub fn set_index(&self, index: i32) {
        *self.index.borrow_mut() = index;
    }

    pub fn get_index(&self) -> i32 {
        *self.index.borrow()
    }

    pub fn to_bytes(&self) -> [u8; 48] {
        let mut bytes = [0u8; 48];
        unsafe {
            let albedo_bytes: [u8; 12] = transmute(self.albedo.to32());
            let emission_bytes: [u8; 12] = transmute(self.emission.to32());
            let metallic_bytes = self.metallic.to_f32().unwrap().to_le_bytes();
            let roughness_bytes = self.roughness.to_f32().unwrap().to_le_bytes();
            let ao_bytes = self.ao.to_f32().unwrap().to_le_bytes();
            bytes[0..12].copy_from_slice(&albedo_bytes);
            bytes[16..28].copy_from_slice(&emission_bytes);
            bytes[32..36].copy_from_slice(&metallic_bytes);
            bytes[36..40].copy_from_slice(&roughness_bytes);
            bytes[40..44].copy_from_slice(&ao_bytes);
        }
        bytes
    }
}
