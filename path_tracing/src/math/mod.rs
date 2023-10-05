use std::f32::consts::PI;

pub mod vector;

pub struct Math;
impl Math {
    pub fn radian(degree: f32) -> f32 {
        return degree * PI / 180.0
    }

    pub fn degree(radian: f32) -> f32 {
        return radian / PI * 180.0
    }
}