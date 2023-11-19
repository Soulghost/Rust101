use crate::math::Vector3f;

pub struct Ray {
    pub origin: Vector3f,
    pub direction: Vector3f,
    pub t: f64,
    pub t_min: f64,
    pub t_max: f64,
}

impl Ray {
    pub fn new(origin: &Vector3f, direction: &Vector3f, t: f64) -> Ray {
        Ray {
            t_min: 0.0,
            t_max: f64::MAX,
            origin: *origin,
            direction: *direction,
            t,
        }
    }

    pub fn eval(&self, t: f64) -> Vector3f {
        self.origin + self.direction * t
    }
}
