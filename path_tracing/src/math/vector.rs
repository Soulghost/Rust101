pub struct Vector3f {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Vector3f {
    pub fn zero() -> Vector3f {
        Vector3f { x: 0.0, y: 0.0, z: 0.0 }
    }
}