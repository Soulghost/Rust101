use std::{f64::EPSILON, fmt::Display};

use crate::{math::vector::Vector3f, domain::domain::{Axis, Ray}};
pub struct Bounds3 {
    pub p_min: Vector3f,
    pub p_max: Vector3f
}

impl Bounds3 {
    pub fn zero() -> Bounds3 {
        Bounds3 { p_min: Vector3f::zero(), p_max: Vector3f::zero() }
    }

    pub fn from_points(p1: &Vector3f, p2: &Vector3f) -> Bounds3 {
        Bounds3 { 
            p_min: Vector3f::new(
                f64::min(p1.x, p2.x),
                f64::min(p1.y, p2.y),
                f64::min(p1.z, p2.z),
            ), 
            p_max: Vector3f::new(
                f64::max(p1.x, p2.x),
                f64::max(p1.y, p2.y),
                f64::max(p1.z, p2.z),
            ) 
        }
    }

    pub fn union(&mut self, b: &Bounds3) {
        self.p_min = Vector3f::min(&self.p_min, &b.p_min);
        self.p_max = Vector3f::max(&self.p_max, &b.p_max);
    }

    pub fn union_point(&mut self, p: &Vector3f) {
        self.p_min = Vector3f::min(&self.p_min, p);
        self.p_max = Vector3f::max(&self.p_max, p);
    }

    pub fn center(&self) -> Vector3f {
        &self.p_min * 0.5 + &self.p_max * 0.5
    }

    pub fn diagonal(&self) -> Vector3f {
        &self.p_max - &self.p_min
    }

    pub fn max_extent_axis(&self) -> Axis {
        let d = self.diagonal();
        if d.x > d.y && d.x > d.z {
            return Axis::X;
        } else if d.y > d.z {
            return Axis::Y;
        }
        Axis::Z
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
        let is_dir_neg = [
            ray.direction.x >= 0.0,
            ray.direction.y >= 0.0,
            ray.direction.z >= 0.0
        ];
        let origin = &ray.origin;
        let p_min = &self.p_min;
        let p_max = &self.p_max;
        let t_min = &(p_min - origin) * &inv_dir;
        let t_max = &(p_max - origin) * &inv_dir;
        let mut t_enter3 = Vector3f::zero();
        let mut t_exit3 = Vector3f::zero();
        t_enter3.x = if is_dir_neg[0] { t_min.x } else { t_max.x };
        t_enter3.y = if is_dir_neg[1] { t_min.y } else { t_max.y };
        t_enter3.z = if is_dir_neg[2] { t_min.z } else { t_max.z };

        t_exit3.x = if !is_dir_neg[0] { t_min.x } else { t_max.x };
        t_exit3.y = if !is_dir_neg[1] { t_min.y } else { t_max.y };
        t_exit3.z = if !is_dir_neg[2] { t_min.z } else { t_max.z };

        let t_enter = f64::max(t_enter3.x, f64::max(t_enter3.y, t_enter3.z));
        let t_exit = f64::min(t_exit3.x, f64::min(t_exit3.y, t_exit3.z));
        t_exit >= t_enter && t_exit >= 0.0
    }

}

impl Clone for Bounds3 {
    fn clone(&self) -> Self {
        Bounds3 { p_min: self.p_min.clone(), p_max: self.p_max.clone() }
    }
}

impl Display for Bounds3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(min={}, max={})", self.p_min, self.p_max)
    }
}