use std::{
    f64::{consts::PI, EPSILON},
    rc::Rc,
};

use crate::{
    math::{lerp, Vector3f},
    sdf::HitResult,
};

pub fn pbr_lighting<'a>(
    hit: &'a HitResult<'a>,
    view: &Vector3f,
    normal: &Vector3f,
    light: &Vector3f,
    light_intensity: &Vector3f,
    // trick for ground
    replace_albedo: Option<Vector3f>,
) -> Vector3f {
    assert!(hit.shape_op.is_some());
    let op = hit.shape_op.unwrap();
    let material = Rc::clone(&op.material);
    let albedo = if let Some(value) = replace_albedo {
        value
    } else {
        material.albedo
    };
    let ambient = &Vector3f::scalar(0.03) * &albedo * (1.0 - material.ao);
    let f0 = lerp(Vector3f::scalar(0.04), albedo, material.metallic);

    // FIXME: multi lights
    let half = ((view + light) * 0.5).normalize();

    // reflection
    let incident_radiance = light_intensity;
    let ndf = normal_distribution_ggx(normal, &half, material.roughness);
    let g = geometry_simth(normal, view, light, material.roughness);
    let f = fresnel_schlick(max(half.dot(view), 0.0), &f0);
    let ks = f;
    let kd = (Vector3f::scalar(1.0) - ks) * (1.0 - material.metallic);
    let numerator = f * ndf * g;
    let denominator = 4.0 * max(normal.dot(view), 0.0) * max(normal.dot(light), 0.0) + f64::EPSILON;
    let specular = numerator / denominator;
    let diffuse = &albedo * &kd / PI;
    let direct_lighting =
        ambient + &(diffuse + specular) * incident_radiance * max(light.dot(normal), 0.0);
    direct_lighting
}

fn normal_distribution_ggx(normal: &Vector3f, half: &Vector3f, roughness: f64) -> f64 {
    let a = roughness * roughness;
    let a2 = a * a;
    let n_dot_h = max(normal.dot(half), 0.0);
    let n_dot_h2 = n_dot_h * n_dot_h;
    let numerator = a2;
    let mut denominator = n_dot_h2 * (a2 - 1.0) + 1.0;
    denominator = PI * denominator * denominator;
    numerator / denominator
}

fn geometry_schlick_ggx(n_dot_v: f64, roughness: f64) -> f64 {
    let r = roughness + 1.0;
    let k = r * r / 8.0;
    let numerator = n_dot_v;
    let denominator = n_dot_v * (1.0 - k) + k;
    numerator / denominator
}

fn geometry_simth(normal: &Vector3f, view: &Vector3f, light: &Vector3f, roughness: f64) -> f64 {
    let n_dot_v = max(normal.dot(view), 0.0);
    let n_dot_l = max(normal.dot(light), 0.0);
    let ggx1 = geometry_schlick_ggx(n_dot_l, roughness);
    let ggx2 = geometry_schlick_ggx(n_dot_v, roughness);
    ggx1 * ggx2
}

fn fresnel_schlick(cos_theta: f64, f0: &Vector3f) -> Vector3f {
    let f1 = (&Vector3f::scalar(1.0) - f0) * f64::powf(1.0 - cos_theta + EPSILON, 5.0);
    f0 + &f1
}

fn max(a: f64, b: f64) -> f64 {
    f64::max(a, b)
}
