use std::{ops, f32::EPSILON, fmt::Display};

#[derive(Clone)]
pub struct Vector3f {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Vector3f {
    pub fn new(x: f32, y: f32, z: f32) -> Vector3f {
        Vector3f { x, y, z }
    }

    pub fn zero() -> Vector3f {
        Vector3f { x: 0.0, y: 0.0, z: 0.0 }
    }

    pub fn clone(&self) -> Vector3f {
        Vector3f { x: self.x, y: self.y, z: self.z }
    }

    pub fn min(p1: &Vector3f, p2: &Vector3f) -> Vector3f {
        Vector3f { 
            x: f32::min(p1.x, p2.x),
            y: f32::min(p1.y, p2.y),
            z: f32::min(p1.z, p2.z),
        }
    }

    pub fn max(p1: &Vector3f, p2: &Vector3f) -> Vector3f {
        Vector3f { 
            x: f32::max(p1.x, p2.x),
            y: f32::max(p1.y, p2.y),
            z: f32::max(p1.z, p2.z),
        }
    }

    pub fn normalize(&self) -> Vector3f {
        let mag2 = self.x * self.x +
                   self.y * self.y +
                   self.z * self.z;
        if mag2 > EPSILON {
            let inv_mag = 1.0 / f32::sqrt(mag2);
            return self * inv_mag
        } else {
            self.clone()
        }
    }

    pub fn length(&self) -> f32 {
        return f32::sqrt(self.x * self.x +
                         self.y * self.y +
                         self.z * self.z);
    }

    pub fn dot(&self, rhs: &Vector3f) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z 
    }

    pub fn cross(&self, rhs: &Vector3f) -> Vector3f {
        Vector3f { 
            x: self.y * rhs.z - self.z * rhs.y, 
            y: self.z * rhs.x - self.x * rhs.z, 
            z: self.x * rhs.y - self.y * rhs.x
        }
    }

    pub fn distance_sq(&self, rhs: &Vector3f) -> f32 {
        (self.x - rhs.x) * (self.x - rhs.x) +
        (self.y - rhs.y) * (self.y - rhs.y) +
        (self.z - rhs.z) * (self.z - rhs.z)
    }
}


impl<T> ops::Mul<T> for Vector3f 
where 
    f64: From<T>
{
    type Output = Vector3f;

    fn mul(self, rhs: T) -> Self::Output {
        let val = f64::from(rhs) as f32;
        Vector3f {
            x: self.x * val,
            y: self.y * val,
            z: self.z * val
        }
    }
}

impl<T> ops::Mul<T> for &Vector3f 
where 
    f64: From<T>
{
    type Output = Vector3f;

    fn mul(self, rhs: T) -> Self::Output {
        let val = f64::from(rhs) as f32;
        Vector3f {
            x: self.x * val,
            y: self.y * val,
            z: self.z * val
        }
    }
}

impl<T> ops::Div<T> for Vector3f 
where 
    f64: From<T>
{
    type Output = Vector3f;

    fn div(self, rhs: T) -> Self::Output {
        let val = f64::from(rhs) as f32;
        Vector3f {
            x: self.x / val,
            y: self.y / val,
            z: self.z / val
        }
    }
}

impl<T> ops::Div<T> for &Vector3f 
where 
    f64: From<T>
{
    type Output = Vector3f;

    fn div(self, rhs: T) -> Self::Output {
        let val = f64::from(rhs) as f32;
        Vector3f {
            x: self.x / val,
            y: self.y / val,
            z: self.z / val
        }
    }
}

impl ops::Add<Vector3f> for Vector3f {
    type Output = Vector3f;

    fn add(self, rhs: Vector3f) -> Self::Output {
        Vector3f {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z
        }
    }
}

impl<'a> ops::Add for &'a Vector3f {
    type Output = Vector3f; 

    fn add(self, rhs: Self) -> Self::Output {
        Vector3f {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z
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
            z: self.z - rhs.z
        }
    }
}

impl<'a> ops::Sub for &'a Vector3f {
    type Output = Vector3f; 

    fn sub(self, rhs: Self) -> Self::Output {
        Vector3f {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z
        }
    }
}

impl<'a> ops::Mul for &'a Vector3f 
{
    type Output = Vector3f;

    fn mul(self, rhs: Self) -> Self::Output {
        Vector3f {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z
        }
    }
}

impl<'a> ops::Neg for &'a Vector3f {
    type Output = Vector3f;

    fn neg(self) -> Self::Output {
        Vector3f {
            x: -self.x,
            y: -self.y,
            z: -self.z
        }
    }
}

impl Display for Vector3f {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "({}, {}, {})", self.x, self.y, self.z);
    }
}