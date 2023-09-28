use crate::math::vector::Vector3f;
pub struct Bounds3 {
    pub p_min: Vector3f,
    pub p_max: Vector3f
}

impl Bounds3 {
    pub fn zero() -> Bounds3 {
        Bounds3 { p_min: Vector3f::zero(), p_max: Vector3f::zero() }
    }
}