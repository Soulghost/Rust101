use crate::math::vector::Vector3f;
pub struct Bounds3 {
    pub p_min: Vector3f,
    pub p_max: Vector3f
}

impl Bounds3 {
    pub fn zero() -> Bounds3 {
        Bounds3 { p_min: Vector3f::zero(), p_max: Vector3f::zero() }
    }

    pub fn union(&mut self, b: &Bounds3) {
        self.p_min = Vector3f::min(&self.p_min, &b.p_min);
        self.p_max = Vector3f::min(&self.p_max, &b.p_max);
    }
}