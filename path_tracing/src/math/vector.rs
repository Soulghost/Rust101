pub struct Vector3f {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Vector3f {
    pub fn zero() -> Vector3f {
        Vector3f { x: 0.0, y: 0.0, z: 0.0 }
    }

    pub fn min(p1: &Vector3f, p2: &Vector3f) -> Vector3f {
        Vector3f { 
            x: f32::min(p1.x, p2.x),
            y: f32::min(p1.y, p2.y),
            z: f32::min(p1.z, p2.z),
        }
    }
}