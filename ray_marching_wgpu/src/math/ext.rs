use cgmath::Point3;

use super::Vector3f;

impl From<Point3<f32>> for Vector3f {
    fn from(value: Point3<f32>) -> Self {
        Vector3f {
            x: value.x as f64,
            y: value.y as f64,
            z: value.z as f64,
        }
    }
}
