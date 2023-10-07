use std::sync::Arc;

use crate::{math::vector::Vector3f, mesh::object::Object, material::material::Material};

pub enum Axis {
    X,
    Y,
    Z,
    Nil
}

pub struct Ray {
    pub origin: Vector3f,
    pub direction: Vector3f,
    pub t: f64,
    pub t_min: f64,
    pub t_max: f64
}

impl Ray {
    pub fn new(origin: &Vector3f,
               direction: &Vector3f,
               t: f64) -> Ray {
        Ray {
            t_min: 0.0,
            t_max: f64::MAX,
            origin: origin.clone(),
            direction: direction.clone(),
            t
        }
    }

    pub fn eval(&self, t: f64) -> Vector3f {
        return self.origin.clone() + self.direction.clone() * t;
    }
}

pub struct Intersection {
    pub hit: bool,
    pub coords: Vector3f,
    pub tcoords: Vector3f,
    pub normal: Vector3f,
    pub emit: Vector3f,
    pub distance: f32,
    pub obj: Option<Arc<dyn Object>>,
    pub material: Option<Arc<dyn Material>>
}

impl Intersection {
    pub fn new() -> Intersection {
        Intersection {
            hit: false,
            coords: Vector3f::zero(),
            tcoords: Vector3f::zero(),
            normal: Vector3f::zero(),
            emit: Vector3f::zero(),
            distance: f32::MAX,
            obj: None,
            material: None
        }
    }
}