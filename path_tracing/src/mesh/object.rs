use crate::bvh::bounds::Bounds3;

pub trait Object {
    fn get_bounds(&self) -> Bounds3 {
        return Bounds3::zero();
    }

    fn get_area(&self) -> f32 {
        return 0.0;
    }
}