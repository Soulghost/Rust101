use std::sync::Arc;

use crate::{bvh::bounds::Bounds3, domain::domain::{Intersection, Ray}};

pub trait Object : Send + Sync {
    fn get_name(&self) -> String {
        String::from("Object")
    }

    fn get_bounds(&self) -> Bounds3;
    fn get_area(&self) -> f64;
    fn intersect(self: Arc<Self>, ray: &Ray) -> Intersection;
    fn sample(&self) -> (Intersection, f64);
}