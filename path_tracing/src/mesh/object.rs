use crate::{bvh::bounds::Bounds3, domain::domain::{Intersection, Ray}};

pub trait Object {
    fn get_bounds(&self) -> Bounds3 {
        return Bounds3::zero();
    }

    fn get_area(&self) -> f32 {
        return 0.0;
    }

    fn intersect(&self, ray: &Ray) -> Intersection;

    fn sample(&self) -> (Intersection, f32);
}