use crate::math::{max, min, Vector2f};
use crate::{domain::Ray, math::Vector3f};
use cgmath::num_traits::ToPrimitive;
use core::fmt;
use nalgebra::{Rotation3, Vector3};
use std::f64::consts::TAU;
use std::fmt::Display;
use std::mem::transmute;

use super::{Shape, ShapeType};

pub struct Sphere {
    pub center: Vector3f,
    pub radius: f64,
}

impl Shape for Sphere {
    fn shape_type(&self) -> ShapeType {
        ShapeType::Sphere
    }

    fn sdf(&self, p: &Vector3f) -> f64 {
        (&self.center - p).length() - self.radius
    }

    fn to_bytes(&self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        unsafe {
            let center_bytes: [u8; 12] = transmute(self.center.to32());
            let radius_bytes = self.radius.to_f32().unwrap().to_le_bytes();
            bytes[0..12].copy_from_slice(&center_bytes);
            bytes[12..16].copy_from_slice(&radius_bytes);
        }
        bytes
    }
}

impl Display for Sphere {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sphere(c={}, o={})", self.center, self.radius)
    }
}

pub struct Cube {
    pub most_front_up_right: Vector3f,
    pub center: Vector3f,
}

impl Shape for Cube {
    fn shape_type(&self) -> ShapeType {
        ShapeType::Cube
    }

    fn sdf(&self, p: &Vector3f) -> f64 {
        let mut d_abs = p - &self.center;
        d_abs.x = f64::abs(d_abs.x);
        d_abs.y = f64::abs(d_abs.y);
        d_abs.z = f64::abs(d_abs.z);

        let d = d_abs - self.most_front_up_right;
        let mut d_clamped = d;
        d_clamped.x = f64::max(d.x, 0.0);
        d_clamped.y = f64::max(d.y, 0.0);
        d_clamped.z = f64::max(d.z, 0.0);
        d_clamped.length() + f64::min(f64::max(f64::max(d.x, d.y), d.z), 0.0)
    }
}

impl Display for Cube {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cube(c={}, mfur={})",
            self.center, self.most_front_up_right
        )
    }
}

pub struct CubeFrame {
    pub center: Vector3f,
    pub bounds: Vector3f,
    pub thinkness: f64,
}

impl Shape for CubeFrame {
    fn shape_type(&self) -> ShapeType {
        ShapeType::CubeFrame
    }

    fn sdf(&self, p: &Vector3f) -> f64 {
        let mut p = p - &self.center;
        p.x = f64::abs(p.x) - self.bounds.x;
        p.y = f64::abs(p.y) - self.bounds.y;
        p.z = f64::abs(p.z) - self.bounds.z;

        let mut q = p;
        q.x = f64::abs(q.x + self.thinkness) - self.thinkness;
        q.y = f64::abs(q.y + self.thinkness) - self.thinkness;
        q.z = f64::abs(q.z + self.thinkness) - self.thinkness;

        min(
            min(
                Vector3f::max_scalar(&Vector3f::new(p.x, q.y, q.z), 0.0).length()
                    + min(max(p.x, max(q.y, q.z)), 0.0),
                Vector3f::max_scalar(&Vector3f::new(q.x, p.y, q.z), 0.0).length()
                    + min(max(q.x, max(p.y, q.z)), 0.0),
            ),
            Vector3f::max_scalar(&Vector3f::new(q.x, q.y, p.z), 0.0).length()
                + min(max(q.x, max(q.y, q.z)), 0.0),
        )
    }
}

impl Display for CubeFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cube(center={}, bounds={}, thinkness={})",
            self.center, self.bounds, self.thinkness
        )
    }
}

pub struct Torus {
    pub center: Vector3f,
    pub outer_radius: f64,
    pub inner_radius: f64,
}

impl Shape for Torus {
    fn shape_type(&self) -> ShapeType {
        ShapeType::Torus
    }

    fn sdf(&self, p: &Vector3f) -> f64 {
        Vector2f::new(
            Vector2f::new(p.x - self.center.x, p.z - self.center.z).length() - self.outer_radius,
            p.y - self.center.y,
        )
        .length()
            - self.inner_radius
    }
}

impl Display for Torus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Torus(center={}, r0={}, r1={})",
            self.center, self.outer_radius, self.inner_radius
        )
    }
}

pub struct DeathStar {
    pub center: Vector3f,
    pub ra: f64,
    pub rb: f64,
    pub d: f64,
    pub rotate_y: f64,
}

impl Shape for DeathStar {
    fn shape_type(&self) -> ShapeType {
        ShapeType::DeathStar
    }

    fn sdf(&self, p: &Vector3f) -> f64 {
        let p = p - &self.center;
        let p = Vector2f::new(p.x, Vector2f::new(p.y, p.z).length());
        let ra = self.ra;
        let rb = self.rb;
        let d = self.d;
        let a = (ra * ra - rb * rb + d * d) / (2.0 * d);
        let b = f64::sqrt(max(ra * ra - a * a, 0.0));
        if p.x * b - p.y * a > d * max(b - p.y, 0.0) {
            Vector2f::new(p.x - a, p.y - b).length()
        } else {
            max(
                p.length() - ra,
                -(Vector2f::new(p.x - d, p.y).length() - rb),
            )
        }
    }

    fn rotate_ray(&self, ray: &Ray) -> Ray {
        let dir = Vector3::new(ray.direction.x, ray.direction.y, ray.direction.z);
        let rotation = Rotation3::from_euler_angles(0.0, 0.0, self.rotate_y).inverse();
        let dir_a: nalgebra::Matrix<
            f64,
            nalgebra::Const<3>,
            nalgebra::Const<1>,
            nalgebra::ArrayStorage<f64, 3, 1>,
        > = (rotation * dir).normalize();
        Ray::new(&ray.origin, &Vector3f::new(dir_a.x, dir_a.y, dir_a.z), 0.0)
    }
}

impl Display for DeathStar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DeathStar(center={}, ra={}, rb={}, d={})",
            self.center, self.ra, self.rb, self.d
        )
    }
}

pub struct Helix {
    pub center: Vector3f,
    pub fr: f64,
    pub r1: f64,
    pub r2: f64,
}

impl Shape for Helix {
    fn shape_type(&self) -> ShapeType {
        ShapeType::Helix
    }

    fn sdf(&self, p: &Vector3f) -> f64 {
        let p = p - &self.center;
        let n_line = Vector2f::new(self.fr, TAU * self.r1);
        let p_line = Vector2f::new(n_line.y, -n_line.x);
        let repeat = n_line.x * n_line.y;
        let pc = Vector2f::new(p.x, self.r1 * f64::atan2(p.y, p.z));
        let mut pp = Vector2f::new(pc.dot(&p_line), pc.dot(&n_line));
        pp.x = f64::round(pp.x / repeat) * repeat;
        let qc_num_1 = Vector2f::new(n_line.x * pp.y, n_line.y * pp.y);
        let qc_num_2 = Vector2f::new(p_line.x * pp.x, p_line.y * pp.x);
        let qc_num = Vector2f::new(qc_num_1.x + qc_num_2.x, qc_num_1.y + qc_num_2.y);
        let qc_denom = n_line.dot(&n_line);
        let qc = Vector2f::new(qc_num.x / qc_denom, qc_num.y / qc_denom / self.r1);
        let q = Vector3f::new(qc.x, f64::sin(qc.y) * self.r1, f64::cos(qc.y) * self.r1);
        (p - q).length() - self.r2 - 0.0001
    }
}

impl Display for Helix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Helix(center={}, fr={}, r1={}, r2={})",
            self.center, self.fr, self.r1, self.r2
        )
    }
}
