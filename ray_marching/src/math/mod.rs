use std::{
    f64::consts::PI,
    fmt::Display,
    ops,
    ops::{Add, Mul},
};

#[derive(Copy, Clone)]
pub struct Vector3f {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3f {
    pub fn new(x: f64, y: f64, z: f64) -> Vector3f {
        Vector3f { x, y, z }
    }

    pub fn scalar(v: f64) -> Vector3f {
        Vector3f { x: v, y: v, z: v }
    }

    pub fn zero() -> Vector3f {
        Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn min(p1: &Vector3f, p2: &Vector3f) -> Vector3f {
        Vector3f {
            x: f64::min(p1.x, p2.x),
            y: f64::min(p1.y, p2.y),
            z: f64::min(p1.z, p2.z),
        }
    }

    pub fn max(p1: &Vector3f, p2: &Vector3f) -> Vector3f {
        Vector3f {
            x: f64::max(p1.x, p2.x),
            y: f64::max(p1.y, p2.y),
            z: f64::max(p1.z, p2.z),
        }
    }

    pub fn max_scalar(p1: &Vector3f, val: f64) -> Vector3f {
        Vector3f {
            x: f64::max(p1.x, val),
            y: f64::max(p1.y, val),
            z: f64::max(p1.z, val),
        }
    }

    pub fn normalize(&self) -> Vector3f {
        let mag2 = self.x * self.x + self.y * self.y + self.z * self.z;
        if mag2 > f64::EPSILON {
            let inv_mag = 1.0 / f64::sqrt(mag2);
            self * inv_mag
        } else {
            *self
        }
    }

    pub fn length(&self) -> f64 {
        f64::sqrt(self.x * self.x + self.y * self.y + self.z * self.z)
    }

    pub fn dot(&self, rhs: &Vector3f) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn cross(&self, rhs: &Vector3f) -> Vector3f {
        Vector3f {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    pub fn distance_sq(&self, rhs: &Vector3f) -> f64 {
        (self.x - rhs.x) * (self.x - rhs.x)
            + (self.y - rhs.y) * (self.y - rhs.y)
            + (self.z - rhs.z) * (self.z - rhs.z)
    }
}

impl<T> ops::Mul<T> for Vector3f
where
    f64: From<T>,
{
    type Output = Vector3f;

    fn mul(self, rhs: T) -> Self::Output {
        let val = f64::from(rhs);
        Vector3f {
            x: self.x * val,
            y: self.y * val,
            z: self.z * val,
        }
    }
}

impl<T> ops::Mul<T> for &Vector3f
where
    f64: From<T>,
{
    type Output = Vector3f;

    fn mul(self, rhs: T) -> Self::Output {
        let val = f64::from(rhs);
        Vector3f {
            x: self.x * val,
            y: self.y * val,
            z: self.z * val,
        }
    }
}

impl<T> ops::Div<T> for Vector3f
where
    f64: From<T>,
{
    type Output = Vector3f;

    fn div(self, rhs: T) -> Self::Output {
        let val = f64::from(rhs);
        Vector3f {
            x: self.x / val,
            y: self.y / val,
            z: self.z / val,
        }
    }
}

impl<T> ops::Div<T> for &Vector3f
where
    f64: From<T>,
{
    type Output = Vector3f;

    fn div(self, rhs: T) -> Self::Output {
        let val = f64::from(rhs);
        Vector3f {
            x: self.x / val,
            y: self.y / val,
            z: self.z / val,
        }
    }
}

impl ops::Add<Vector3f> for Vector3f {
    type Output = Vector3f;

    fn add(self, rhs: Vector3f) -> Self::Output {
        Vector3f {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<'a> ops::Add for &'a Vector3f {
    type Output = Vector3f;

    fn add(self, rhs: Self) -> Self::Output {
        Vector3f {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl ops::AddAssign for Vector3f {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl ops::Sub<Vector3f> for Vector3f {
    type Output = Vector3f;

    fn sub(self, rhs: Vector3f) -> Self::Output {
        Vector3f {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<'a> ops::Sub for &'a Vector3f {
    type Output = Vector3f;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector3f {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::Sub<f64> for Vector3f {
    type Output = Vector3f;

    fn sub(self, rhs: f64) -> Self::Output {
        Vector3f {
            x: self.x - rhs,
            y: self.y - rhs,
            z: self.z - rhs,
        }
    }
}

impl<'a> ops::Mul for &'a Vector3f {
    type Output = Vector3f;

    fn mul(self, rhs: Self) -> Self::Output {
        Vector3f {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl<'a> ops::Neg for &'a Vector3f {
    type Output = Vector3f;

    fn neg(self) -> Self::Output {
        Vector3f {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Display for Vector3f {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

pub struct Math;
impl Math {
    pub fn radian(degree: f64) -> f64 {
        degree * PI / 180.0
    }

    pub fn degree(radian: f64) -> f64 {
        radian / PI * 180.0
    }
}

pub struct Vector2f {
    pub x: f64,
    pub y: f64,
}

impl Vector2f {
    pub fn new(x: f64, y: f64) -> Vector2f {
        Vector2f { x, y }
    }

    pub fn length(&self) -> f64 {
        f64::sqrt(self.x * self.x + self.y * self.y)
    }

    pub fn dot(&self, rhs: &Vector2f) -> f64 {
        self.x * rhs.x + self.y * rhs.y
    }
}

pub fn lerp<T>(x: T, y: T, a: f64) -> T
where
    T: Mul<f64, Output = T> + Add<Output = T> + Copy,
{
    x * (1.0 - a) + y * a
}

pub fn min(a: f64, b: f64) -> f64 {
    f64::min(a, b)
}

pub fn max(a: f64, b: f64) -> f64 {
    f64::max(a, b)
}
