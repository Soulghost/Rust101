use std::f32::consts::PI;
use rand::Rng;
use rand_distr::Uniform;

pub mod vector;

pub struct Math;
impl Math {
    pub fn radian(degree: f32) -> f32 {
        return degree * PI / 180.0
    }

    pub fn degree(radian: f32) -> f32 {
        return radian / PI * 180.0
    }

    pub fn sample_uniform_distribution(low: f32, high: f32) -> f32 {
        let uni = Uniform::new(low, high);
        let mut rng = rand::thread_rng();
        return rng.sample(uni);
    }
}