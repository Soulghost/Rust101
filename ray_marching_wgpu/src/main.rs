use std::rc::Rc;

use material::PBRMaterial;
use math::Vector3f;
use pipeline::State;
use sdf::{primitive::Sphere, Scene};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

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
    // window.set_inner_size(LogicalSize::new(500, 500));
    let mut state = State::new(window).await;
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
                let scene = Scene::new(0, 0, 0.0, 0, Vector3f::zero());
                let purper_material = Rc::new(PBRMaterial {
                    albedo: Vector3f::new(235.0 / 255.0, 81.0 / 255.0, 1.0),
                    emission: Vector3f::zero(),
                    metallic: 0.0,
                    roughness: 0.8,
                    ao: 0.05,
                });
                let node = scene.add_leaf_node(
                    Box::new(Sphere {
                        center: Vector3f::new(0.0, 0.0, 0.0),
                        radius: 0.5,
                    }),
                    purper_material,
                );
                scene.add_root_node(node);
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
