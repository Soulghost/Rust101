use crate::math::Vector3f;
use cgmath::num_traits::ToPrimitive;
use core::fmt;
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

    fn to_bytes(&self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        unsafe {
            let center_bytes: [u8; 12] = transmute(self.center.to32());
            let most_front_up_right_bytes: [u8; 12] = transmute(self.most_front_up_right.to32());
            bytes[0..12].copy_from_slice(&center_bytes);
            bytes[12..24].copy_from_slice(&most_front_up_right_bytes);
        }
        bytes
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

pub struct VolumetricCloud {
    pub most_front_up_right: Vector3f,
    pub center: Vector3f,
    // FIXME: texture
}

impl Shape for VolumetricCloud {
    fn shape_type(&self) -> ShapeType {
        ShapeType::VolumetricCloud
    }

    fn to_bytes(&self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        unsafe {
            let center_bytes: [u8; 12] = transmute(self.center.to32());
            let most_front_up_right_bytes: [u8; 12] = transmute(self.most_front_up_right.to32());
            bytes[0..12].copy_from_slice(&center_bytes);
            bytes[12..24].copy_from_slice(&most_front_up_right_bytes);
        }
        bytes
    }
}

impl Display for VolumetricCloud {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "VolumetricCloud(c={}, mfur={})",
            self.center, self.most_front_up_right
        )
    }
}
