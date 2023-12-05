use std::{rc::Rc, time::Instant};

use material::PBRMaterial;
use math::Vector3f;
use pipeline::State;
use sdf::{primitive::Sphere, Scene};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::sdf::primitive::Cube;

pub mod domain;
pub mod material;
pub mod math;
pub mod node;
pub mod pipeline;
pub mod renderer;
pub mod sdf;

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut state = State::new(window).await;

    // test animation
    let max_distance = 1.25;
    let mut is_up = true;
    let speed = 1.0;
    let mut prev_time = Instant::now();
    let mut child_position = Vector3f::new(0.5, 0.0, -0.25);
    child_position = Vector3f::new(-0.82517262369708, 0.0, 0.3386258631184854);
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &mut so w have to dereference it twice
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                let now_time = Instant::now();
                let delta_time = now_time.duration_since(prev_time).as_secs_f32();
                if delta_time < 1.0 / 60.0 {
                    // skip this frame
                    return;
                }
                prev_time = now_time;
                let move_vec = if is_up {
                    Vector3f::new(-0.5, 0.0, 0.25).normalize()
                } else {
                    Vector3f::new(0.5, 0.0, -0.25).normalize()
                };
                child_position += move_vec * (speed * delta_time);
                if child_position.length() > max_distance {
                    is_up = !is_up;
                }
                println!(
                    "delta_time: {}, fps: {}, child_pos: {}",
                    delta_time,
                    1.0 / delta_time,
                    child_position
                );

                let scene = Scene::new(0, 0, 0.0, 0, Vector3f::zero());
                let purper_material = Rc::new(PBRMaterial::new(
                    Vector3f::new(235.0 / 255.0, 81.0 / 255.0, 1.0),
                    Vector3f::zero(),
                    0.0,
                    0.95,
                    0.05,
                ));
                let metal_material = Rc::new(PBRMaterial::new(
                    Vector3f::new(235.0 / 255.0, 232.0 / 255.0, 1.0),
                    Vector3f::zero(),
                    0.85,
                    0.30,
                    0.025,
                ));
                let ground_material = Rc::new(PBRMaterial::new(
                    Vector3f::new(-1.0, -1.0, -1.0),
                    Vector3f::zero(),
                    0.0,
                    1.0,
                    0.0,
                ));
                let child_sphere = scene.add_leaf_node(
                    Box::new(Sphere {
                        center: child_position,
                        radius: 0.25,
                    }),
                    Rc::clone(&metal_material),
                );
                let root_sphere = scene.add_node(
                    Box::new(Sphere {
                        center: Vector3f::new(0.0, 0.0, 0.0),
                        radius: 0.5,
                    }),
                    Rc::clone(&metal_material),
                    sdf::ShapeOpType::SmoothUnion,
                    Some(child_sphere),
                );

                let ground_node = scene.add_leaf_node(
                    Box::new(Cube {
                        center: Vector3f::new(0.0, -4.0, 0.0),
                        most_front_up_right: Vector3f::new(15.0, 0.25, 15.0),
                    }),
                    Rc::clone(&ground_material),
                );

                // let emission_cube = scene.add_leaf_node(
                //     Box::new(Cube {
                //         center: Vector3f::new(-1.0, 0.0, -0.5),
                //         // center: Vector3f::new(0.0, 0.0, 0.0),
                //         most_front_up_right: Vector3f::new(0.25, 0.25, 0.25),
                //     }),
                //     Rc::clone(&purper_material),
                // );
                scene.add_root_node(root_sphere);
                scene.add_root_node(ground_node);
                // scene.add_root_node(emission_cube);
                state.update(&scene);

                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.resize(state.size)
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // We're ignoring timeouts
                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                state.window().request_redraw();
            }
            _ => {}
        }
    });
}

fn main() {
    pollster::block_on(run());
}
