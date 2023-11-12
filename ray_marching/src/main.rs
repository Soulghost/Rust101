#![feature(trait_upcasting)]
use std::{any::Any, time::Instant};

use math::Vector3f;
use minifb::{Key, Window, WindowOptions};
use sdf::{Scene, Sphere};

use crate::renderer::{framebuffer::FrameBuffer, rendering::Renderer};

pub mod domain;
pub mod math;
pub mod renderer;
pub mod sdf;

fn render_and_dump() {
    let scene = Scene::new(
        720,
        720,
        45.0,
        1,
        Vector3f::new(0.235294, 0.67451, 0.843137),
    );

    // add sphere sdf <s0 U s0_leaf>
    let sphere0_leaf = scene.add_leaf_node(Box::new(Sphere {
        center: Vector3f::new(0.5, -2.5 * f64::cos(0.1), 0.0),
        radius: 0.5,
    }));
    let sphere0 = scene.add_node(
        Box::new(Sphere {
            center: Vector3f::new(0.8, -2.5 * f64::cos(0.5), -0.25),
            radius: 0.35,
        }),
        sdf::ShapeOpType::Union,
        Some(sphere0_leaf),
    );
    scene.add_root_node(sphere0);

    // renderer
    let mut renderer = Renderer::new();
    let fbo = FrameBuffer::new(scene.width, scene.height);
    renderer.fbo = Some(fbo);

    println!("[Main] start rendering...");
    renderer
        .render(Vector3f::new(0.5, -2.5, -5.0), &scene, false)
        .unwrap_or_else(|err| {
            panic!("[Main] renderer error {}", err);
        });
    println!("[Main] end rendering...");

    let fbo = renderer.fbo.as_mut().unwrap();
    let rt = fbo.get_render_target();
    rt.dump_to_file("out/result.ppm").unwrap_or_else(|err| {
        panic!("[Main] dump rt to file error {}", err);
    });
}

fn realtime_rendering() {
    let width = 400;
    let height = 400;
    let mut window = Window::new("Ray Marching", width, height, WindowOptions::default())
        .unwrap_or_else(|e| {
            panic!("[Main] cannot create native window {}", e);
        });
    let mut f = 0;
    let timer = Instant::now();
    let mut t0 = timer.elapsed().as_secs_f64();
    let mut sphere0_center = Vector3f::new(0.5, -2.5 * f64::cos(0.1), 0.0);

    let scene = Scene::new(
        width as u32,
        height as u32,
        45.0,
        1,
        Vector3f::new(0.235294, 0.67451, 0.843137),
    );
    // add sphere sdf <s0 U s0_leaf>
    let sphere0_leaf: &sdf::ShapeOp<'_> = scene.add_leaf_node(Box::new(Sphere {
        center: sphere0_center.clone(),
        radius: 0.5,
    }));
    let sphere0 = scene.add_node(
        Box::new(Sphere {
            center: Vector3f::new(0.8, -2.5 * f64::cos(0.2), -0.25),
            radius: 0.35,
        }),
        sdf::ShapeOpType::Union,
        Some(sphere0_leaf),
    );
    scene.add_root_node(sphere0);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let delta_time = timer.elapsed().as_secs_f64() - t0;
        t0 = timer.elapsed().as_secs_f64();

        sphere0_center += Vector3f::new(delta_time * 0.1, 0.0, 0.0);

        // renderer
        let mut renderer = Renderer::new();
        let fbo = FrameBuffer::new(scene.width, scene.height);
        renderer.fbo = Some(fbo);

        renderer
            .render(Vector3f::new(0.7, -2.5, -3.0), &scene, true)
            .unwrap_or_else(|err| {
                panic!("[Main] renderer error {}", err);
            });

        let fbo = renderer.fbo.as_mut().unwrap();
        let rt = fbo.get_render_target();
        window
            .update_with_buffer(&rt.get_buffer(), width, height)
            .unwrap();
        println!("draw f{}", f);
        f += 1;
    }
}

fn main() {
    realtime_rendering();
}
