use std::f32::EPSILON;

use crate::{math::vector::Vector3f, domain::domain::{Axis, Ray}};
pub struct Bounds3 {
    pub p_min: Vector3f,
    pub p_max: Vector3f
}

impl Bounds3 {
    pub fn zero() -> Bounds3 {
        Bounds3 { p_min: Vector3f::zero(), p_max: Vector3f::zero() }
    }

    pub fn clone(&self) -> Bounds3 {
        Bounds3 { p_min: self.p_min.clone(), p_max: self.p_max.clone() }
    }

    pub fn union(&mut self, b: &Bounds3) {
        self.p_min = Vector3f::min(&self.p_min, &b.p_min);
        self.p_max = Vector3f::max(&self.p_max, &b.p_max);
    }

    pub fn union_point(&mut self, p: &Vector3f) {
        self.p_min = Vector3f::min(&self.p_min, &p);
        self.p_max = Vector3f::max(&self.p_max, &p);
    }

    pub fn center(&self) -> Vector3f {
        return self.p_min.clone() * 0.5 + self.p_max.clone() * 0.5;
    }

    pub fn diagonal(&self) -> Vector3f {
        return self.p_max.clone() - self.p_min.clone();
    }

    pub fn max_extent_axis(&self) -> Axis {
        let d = self.diagonal();
        if d.x > d.y && d.x > d.z {
            return Axis::X;
        } else if d.y > d.z {
            return Axis::Y;
        }
        return Axis::Z;
    }

    pub fn union2(a: &Bounds3, b: &Bounds3) -> Bounds3 {
        Bounds3 {
            p_min: Vector3f::min(&a.p_min, &b.p_min),
            p_max: Vector3f::max(&a.p_max, &b.p_max),
        }
    }

    pub fn intersect(&self, ray: &Ray) -> bool {
        let inv_dir = Vector3f::new(
            1.0 / (ray.direction.x + EPSILON),
            1.0 / (ray.direction.y + EPSILON),
            1.0 / (ray.direction.z + EPSILON)
        );
        let is_dir_neg = vec![
            ray.direction.x >= EPSILON,
            ray.direction.y >= EPSILON,
            ray.direction.z >= EPSILON
        ];
        let origin = &ray.origin;
        let p_min = &self.p_min;
        let p_max = &self.p_max;
        let t_min = &(p_min - origin) * &inv_dir;
        let t_max = &(p_max - origin) * &inv_dir;
        let mut t_enter3 = Vector3f::zero();
        let mut t_exit3 = Vector3f::zero();
        t_enter3.x = if is_dir_neg[0] { t_min.x } else { t_max.x };
        t_enter3.y = if is_dir_neg[0] { t_min.y } else { t_max.y };
        t_enter3.z = if is_dir_neg[0] { t_min.z } else { t_max.z };

        t_exit3.x = if !is_dir_neg[0] { t_min.x } else { t_max.x };
        t_exit3.y = if !is_dir_neg[0] { t_min.y } else { t_max.y };
        t_exit3.z = if !is_dir_neg[0] { t_min.z } else { t_max.z };

        let t_enter = f32::max(t_enter3.x, f32::max(t_enter3.y, t_enter3.z));
        let t_exit = f32::min(t_exit3.x, f32::min(t_exit3.y, t_exit3.z));
        return t_exit >= t_enter && t_exit >= EPSILON;
    }

}