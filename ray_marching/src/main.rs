#![feature(trait_upcasting)]
use std::rc::Rc;

use material::PBRMaterial;
use math::Vector3f;
use minifb::{Key, Window, WindowOptions};
use sdf::{
    primitive::{Cube, Helix, Sphere, Torus},
    Scene,
};

use crate::renderer::{framebuffer::FrameBuffer, rendering::Renderer};

pub mod domain;
pub mod material;
pub mod math;
pub mod renderer;
pub mod sdf;

fn render(show_window: bool) {
    let dpi = 1;
    let width = 720 * dpi;
    let height = 405 * dpi;
    let mut window = Window::new("Ray Marching", width, height, WindowOptions::default())
        .unwrap_or_else(|e| {
            panic!("[Main] cannot create native window {}", e);
        });
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    // rotation
    let eye = Vector3f::new(-0.3, 4.0, -9.5);
    let rotation = Vector3f::new(32.0, 0.0, 0.0);
    let scene = Scene::new(
        width as u32,
        height as u32,
        60.0,
        1,
        Vector3f::new(0.235294, 0.67451, 0.843137),
    );

    // Cube Frame
    add_models_to_scene(&scene);

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

    // show in window
    if show_window {
        let buffer = &rt.get_buffer(false);
        while window.is_open() && !window.is_key_down(Key::Escape) {
            window.update_with_buffer(buffer, width, height).unwrap();
        }
    }

    // dump to file
    rt.dump_to_file("out/result.ppm").unwrap_or_else(|err| {
        panic!("[Main] dump rt to file error {}", err);
    });
}

fn add_models_to_scene<'a>(scene: &'a Scene<'a>) {
    // material
    let ground_material = Rc::new(PBRMaterial {
        albedo: Vector3f::new(1.0, 1.0, 1.0) * 1.0,
        emission: Vector3f::zero(),
        metallic: 0.0,
        roughness: 0.95,
        ao: 0.0,
    });
    let purper_material = Rc::new(PBRMaterial {
        albedo: Vector3f::new(235.0 / 255.0, 81.0 / 255.0, 1.0),
        emission: Vector3f::zero(),
        metallic: 0.0,
        roughness: 0.8,
        ao: 0.05,
    });
    let metal_material = Rc::new(PBRMaterial {
        albedo: Vector3f::new(0.95, 0.98, 0.98),
        emission: Vector3f::zero(),
        metallic: 0.85,
        roughness: 0.25,
        ao: 0.05,
    });
    let metal_frame_material = Rc::new(PBRMaterial {
        albedo: Vector3f::new(0.95, 0.95, 0.95),
        emission: Vector3f::zero(),
        metallic: 0.5,
        roughness: 0.5,
        ao: 0.1,
    });

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

    // Sphere
    let sub_sphere = scene.add_leaf_node(
        Box::new(Sphere {
            center: Vector3f::new(0.0, 2.0, -5.6),
            radius: 0.5,
        }),
        Rc::clone(&purper_material),
    );
    let sphere = scene.add_node(
        Box::new(Sphere {
            center: Vector3f::new(0.0, 1.65, -5.6),
            radius: 0.8,
        }),
        Rc::clone(&purper_material),
        sdf::ShapeOpType::Subtraction,
        Some(sub_sphere),
    );
    scene.add_root_node(sphere);

    // Helix
    let helix = scene.add_leaf_node(
        Box::new(Helix {
            center: Vector3f::new(3.4, 1.5, 3.0),
            fr: 1.25,
            r1: 0.8,
            r2: 0.25,
        }),
        Rc::clone(&metal_frame_material),
    );
    scene.add_root_node(helix);
}

fn main() {
    render(true);
}
