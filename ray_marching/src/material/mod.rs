use crate::math::Vector3f;

pub struct PBRMaterial {
    pub kd: Vector3f,
    pub emission: Vector3f,
    pub metalness: f64,
    pub roughness: f64,
}
