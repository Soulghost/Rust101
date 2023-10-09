use std::f64::consts::PI;
use rand::Rng;
use rand_distr::Uniform;

pub mod vector;

pub struct Math;
impl Math {
    pub fn radian(degree: f64) -> f64 {
        return degree * PI / 180.0
    }

    pub fn degree(radian: f64) -> f64 {
        return radian / PI * 180.0
    }

    pub fn sample_uniform_distribution(low: f64, high: f64) -> f64 {
        let uni = Uniform::new(low, high);
        let mut rng = rand::thread_rng();
        return rng.sample(uni);
    }
}