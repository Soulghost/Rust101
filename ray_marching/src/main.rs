#![feature(trait_upcasting)]
use std::{rc::Rc, time::Instant};

use material::PBRMaterial;
use math::Vector3f;
use minifb::{Key, Window, WindowOptions};
use sdf::{Scene, Sphere};

use crate::{
    renderer::{framebuffer::FrameBuffer, rendering::Renderer},
    sdf::{Cube, Torus},
};

pub mod domain;
pub mod material;
pub mod math;
pub mod renderer;
pub mod sdf;

// fn render_and_dump() {
//     let scene = Scene::new(
//         720,
//         720,
//         45.0,
//         1,
//         Vector3f::new(0.235294, 0.67451, 0.843137),
//     );

//     // add sphere sdf <s0 U s0_leaf>
//     let sphere0_leaf = scene.add_leaf_node(Box::new(Sphere {
//         center: Vector3f::new(0.5, -2.5 * f64::cos(0.1), 0.0),
//         radius: 0.5,
//     }));
//     let sphere0 = scene.add_node(
//         Box::new(Sphere {
//             center: Vector3f::new(0.8, -2.5 * f64::cos(0.5), -0.25),
//             radius: 0.35,
//         }),
//         sdf::ShapeOpType::Union,
//         Some(sphere0_leaf),
//     );
//     // scene.add_root_node(sphere0);

//     // add torus
//     let torus0 = scene.add_leaf_node(Box::new(Torus {
//         center: Vector3f::new(0.5, -2.5 * f64::cos(0.1), 0.0),
//         outer_radius: 2.0,
//         inner_radius: 1.6,
//     }));
//     scene.add_root_node(torus0);

//     // renderer
//     let mut renderer = Renderer::new();
//     let fbo = FrameBuffer::new(scene.width, scene.height);
//     renderer.fbo = Some(fbo);

//     println!("[Main] start rendering...");
//     renderer
//         .render(Vector3f::new(0.5, -2.5, -5.0), &scene, false)
//         .unwrap_or_else(|err| {
//             panic!("[Main] renderer error {}", err);
//         });
//     println!("[Main] end rendering...");

//     let fbo = renderer.fbo.as_mut().unwrap();
//     let rt = fbo.get_render_target();
//     rt.dump_to_file("out/result.ppm").unwrap_or_else(|err| {
//         panic!("[Main] dump rt to file error {}", err);
//     });
// }

fn realtime_rendering() {
    let width = 720;
    let height = 405;
    let mut window = Window::new("Ray Marching", width, height, WindowOptions::default())
        .unwrap_or_else(|e| {
            panic!("[Main] cannot create native window {}", e);
        });
    window.update();
    let mut f = 0;
    let timer = Instant::now();
    let mut t0 = timer.elapsed().as_secs_f64();

    // material
    let ground_material = Rc::new(PBRMaterial {
        kd: Vector3f::new(1.0, 1.0, 1.0) * 1.0,
        emission: Vector3f::zero(),
        metalness: 0.0,
        roughness: 0.95,
    });
    let purper_material = Rc::new(PBRMaterial {
        kd: Vector3f::new(235.0 / 255.0, 81.0 / 255.0, 1.0),
        emission: Vector3f::zero(),
        metalness: 0.0,
        roughness: 0.8,
    });
    let metal_material = Rc::new(PBRMaterial {
        kd: Vector3f::new(0.95, 0.98, 0.98),
        emission: Vector3f::zero(),
        metalness: 0.95,
        roughness: 0.05,
    });

    // rotation
    let eye = Vector3f::new(-0.3, 4.0, -9.5);
    let rotation = Vector3f::new(32.0, 0.0, 0.0);
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let _delta_time = timer.elapsed().as_secs_f64() - t0;
        t0 = timer.elapsed().as_secs_f64();
        // rotation.x -= delta_time;
        // println!("current d {}", rotation.x);

        let scene = Scene::new(
            width as u32,
            height as u32,
            60.0,
            1,
            Vector3f::new(0.235294, 0.67451, 0.843137),
        );

        // Ground
        let ground = scene.add_leaf_node(
            Box::new(Cube {
                center: Vector3f::new(0.0, 0.0, 0.0),
                most_front_up_right: Vector3f::new(15.0, 0.25, 15.0),
            }),
            Rc::clone(&ground_material),
        );
        scene.add_root_node(ground);
        scene.set_ground(ground);

        // Sphere
        let sphere = scene.add_leaf_node(
            Box::new(Sphere {
                center: Vector3f::new(0.0, 1.4, -5.6),
                radius: 1.0,
            }),
            Rc::clone(&purper_material),
        );
        scene.add_root_node(sphere);

        // Torus
        let torus = scene.add_leaf_node(
            Box::new(Torus {
                center: Vector3f::new(-3.2, 1.4, -3.4),
                outer_radius: 1.0,
                inner_radius: 0.55,
            }),
            Rc::clone(&metal_material),
        );
        scene.add_root_node(torus);

        // // add sphere sdf <s0 U s0_leaf>
        // let sphere0_leaf: &sdf::ShapeOp<'_> = scene.add_leaf_node(
        //     Box::new(Sphere {
        //         center: Vector3f::new(1.5, -2.0, 0.0),
        //         radius: 0.55,
        //     }),
        //     Rc::clone(&purper_light_material),
        // );
        // let sphere0 = scene.add_node(
        //     Box::new(Sphere {
        //         center: Vector3f::new(1.8, -1.5, 0.2),
        //         radius: 0.3,
        //     }),
        //     Rc::clone(&purper_light_material),
        //     sdf::ShapeOpType::Union,
        //     Some(sphere0_leaf),
        // );
        // scene.add_root_node(sphere0);

        // renderer
        let mut renderer = Renderer::new();
        let fbo = FrameBuffer::new(scene.width, scene.height);
        renderer.fbo = Some(fbo);

        renderer
            .render(eye, rotation, &scene, true)
            .unwrap_or_else(|err| {
                panic!("[Main] renderer error {}", err);
            });

        let fbo = renderer.fbo.as_mut().unwrap();
        let rt = fbo.get_render_target();
        window
            .update_with_buffer(&rt.get_buffer(false), width, height)
            .unwrap();
        println!("draw f{}", f);
        f += 1;
    }
}

fn main() {
    realtime_rendering();
}
