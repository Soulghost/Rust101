use crate::{math::vector::Vector3f, domain::domain::Axis};
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
        self.p_max = Vector3f::max(&self.p_max, &b.p_max);
    }

    pub fn center(&self) -> Vector3f {
        return self.p_min.clone() * 0.5 + self.p_max.clone() * 0.5;
    }

    pub fn diagonal(&self) -> Vector3f {
        return self.p_max.clone() - self.p_min.clone();
    }

    pub fn max_extent_axis(&self) -> Axis {
        let d = self.diagonal();
        if d.x > d.y && d.x > d.z {
            return Axis::X;
        } else if d.y > d.z {
            return Axis::Y;
        }
        return Axis::Z;
    }

    pub fn union2(a: &Bounds3, b: &Bounds3) -> Bounds3 {
        Bounds3 {
            p_min: Vector3f::min(&a.p_min, &b.p_min),
            p_max: Vector3f::max(&a.p_max, &b.p_max),
        }
    }
}