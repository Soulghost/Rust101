use crate::math::Vector3f;

#[derive(Default, Clone, Copy)]
pub struct Ray {
    pub origin: Vector3f,
    pub direction: Vector3f,
    pub t: f64,
    pub t_min: f64,
    pub t_max: f64,
}

impl From<Ray> for [[f32; 4]; 2] {
    fn from(val: Ray) -> Self {
        [val.origin.into(), val.direction.into()]
    }
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

    pub fn create(origin: Vector3f, direction: Vector3f) -> Ray {
        Ray {
            t: 0.0,
            t_min: 0.0,
            t_max: f64::MAX,
            origin,
            direction,
        }
    }

    pub fn zero() -> Ray {
        Ray {
            origin: Vector3f::zero(),
            direction: Vector3f::zero(),
            t: 0.0,
            t_min: 0.0,
            t_max: f64::MAX,
        }
    }

    pub fn eval(&self, t: f64) -> Vector3f {
        self.origin + self.direction * t
    }
}
