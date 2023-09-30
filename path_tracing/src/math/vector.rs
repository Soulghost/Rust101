use std::ops;

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