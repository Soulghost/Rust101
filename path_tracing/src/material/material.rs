use std::f64::{EPSILON, consts::PI};

use crate::math::{vector::Vector3f, Math};

pub trait Material : Send + Sync {
    fn get_albedo(&self) -> Vector3f;
    fn has_emission(&self) -> bool;
    fn get_emission(&self) -> Vector3f;
    fn eval(&self, ws: &Vector3f, wo: &Vector3f, normal: &Vector3f) -> Vector3f;
    fn sample(&self, _wi: &Vector3f, normal: &Vector3f) -> Vector3f {
        let x1 = Math::sample_uniform_distribution(0.0, 1.0);
        let x2 = Math::sample_uniform_distribution(0.0, 1.0);
        let z = f64::abs(1.0 - 2.0 * x1);
        let r = f64::sqrt(1.0 - z * z);
        let phi = 2.0 * PI * x2;
        let local_dir = Vector3f::new(
            r * f64::cos(phi),
            r * f64::sin(phi),
            z
        );
        let c;
        if f64::abs(normal.x) > f64::abs(normal.y) {
            let inv_len = 1.0 / f64::sqrt(normal.x * normal.x + normal.z * normal.z);
            c = Vector3f::new(normal.z * inv_len, 0.0, -normal.x * inv_len);
        } else {
            let inv_len = 1.0 / f64::sqrt(normal.y * normal.y + normal.z * normal.z);
            c = Vector3f::new(0.0, normal.z * inv_len, -normal.y * inv_len);
        }
        let b = c.cross(normal);
        return b * local_dir.x + c * local_dir.y + normal * local_dir.z;
    }

    fn pdf(&self, _wi: &Vector3f, wo: &Vector3f, normal: &Vector3f) -> f64 {
        if wo.dot(normal) > f64::EPSILON {
            return 0.5 / PI;
        } else {
            return 0.0
        }
    }
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

    fn eval(&self, _ws: &Vector3f, wo: &Vector3f, normal: &Vector3f) -> Vector3f {
        let cosalpha = normal.dot(wo);
        if cosalpha > 0.0 {
            return &self.albedo / PI;
        } else {
            return Vector3f::zero();
        }
    }
}