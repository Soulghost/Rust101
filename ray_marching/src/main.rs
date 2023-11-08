use math::Vector3f;
use sdf::{Scene, Sphere};

pub mod march;
pub mod math;
pub mod sdf;

fn main() {
    let scene = Scene::new();

    // add sphere sdf <s0 U s0_leaf>
    let sphere0_leaf = scene.add_leaf_node(Box::new(Sphere {
        center: Vector3f::new(0.5, -2.5 * f64::cos(0.1), 0.0),
        radius: 0.5,
    }));
    let sphere0 = scene.add_node(
        Box::new(Sphere {
            center: Vector3f::new(0.7, -2.5 * f64::cos(0.2), 0.0),
            radius: 0.5,
        }),
        sdf::ShapeOpType::Union,
        Some(sphere0_leaf),
    );
    scene.add_root_node(sphere0);

    let hit = scene.sdf(&Vector3f::zero());
    println!("the hit result is {}", hit);
}
