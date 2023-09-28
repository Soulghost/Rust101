use crate::bvh::bounds::Bounds3;

pub trait Object {
    fn get_bounds(&self) -> Bounds3 {
        return Bounds3::zero();
    }
}