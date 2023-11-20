use crate::math::Vector3f;

pub mod pbr;

pub struct PBRMaterial {
    pub albedo: Vector3f,
    pub emission: Vector3f,
    pub metallic: f64,
    pub roughness: f64,
    pub ao: f64,
}
