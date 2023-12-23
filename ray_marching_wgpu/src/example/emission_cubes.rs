use std::{future::Future, pin::Pin, rc::Rc, time::Instant};

use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::{
    material::PBRMaterial,
    math::{lerp, Vector3f},
    node::camera::{Camera, CameraController},
    pipeline::State,
    sdf::{
        self,
        primitive::{Cube, Sphere},
        DirectionalLight, Scene, ShapeOp,
    },
};

use super::Application;

pub struct EmissionCubeApp {}

impl EmissionCubeApp {
    pub async fn main() {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();
        window.set_inner_size(PhysicalSize::new(1600, 900));
        let win_size = window.inner_size();

        let mut state = State::new(window).await;
        let mut prev_time = Instant::now();
        let mut elpased_time: f32 = 0.0;
        let mut camera = Camera {
            screen_size: (win_size.width as f32, win_size.height as f32).into(),
            eye: (0.0, 1.0, -6.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: win_size.width as f32 / win_size.height as f32,
            fovy: 60.0,
            znear: 0.1,
            zfar: 100.0,
        };
        let mut camera_controller = CameraController::new(0.2);
        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == state.window().id() => {
                    if !camera_controller.process_events(event) {
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
                                camera.screen_size =
                                    (physical_size.width as f32, physical_size.height as f32)
                                        .into();
                                camera.aspect =
                                    physical_size.width as f32 / physical_size.height as f32;
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
                    elpased_time += delta_time;
                    prev_time = now_time;
                    let window_title = format!("fps: {:.0}", 1.0 / delta_time);
                    state.window.set_title(&window_title);
                    camera_controller.update_camera(&mut camera);
                    let scene = Scene::new(
                        win_size.width,
                        win_size.height,
                        camera,
                        Vector3f::new(0.235294, 0.67451, 0.843137) * 0.02,
                        DirectionalLight {
                            direction: Vector3f::new(0.32, -0.77, 0.56),
                            color: Vector3f::new(1.0, 1.0, 1.0) * 1.0,
                        },
                    );
                    let metal_material = Rc::new(PBRMaterial::new(
                        Vector3f::new(235.0 / 255.0, 232.0 / 255.0, 1.0),
                        Vector3f::zero(),
                        0.85,
                        0.30,
                        0.025,
                    ));
                    let rough_material = Rc::new(PBRMaterial::new(
                        Vector3f::new(246.0 / 255.0, 247.0 / 255.0, 102.0 / 255.0),
                        Vector3f::zero(),
                        0.0,
                        0.95,
                        0.025,
                    ));
                    let ground_material = Rc::new(PBRMaterial::new(
                        Vector3f::new(-1.0, -1.0, -1.0),
                        Vector3f::zero(),
                        0.0,
                        1.0,
                        0.0,
                    ));
                    let root_sphere_left = scene.add_node(
                        Box::new(Sphere {
                            center: Vector3f::new(-3.5, 0.0, -1.2),
                            radius: 0.8,
                        }),
                        Rc::clone(&metal_material),
                        sdf::ShapeOpType::SmoothUnion,
                        None,
                    );
                    let root_sphere_right = scene.add_node(
                        Box::new(Sphere {
                            center: Vector3f::new(3.5, 0.0, -1.2),
                            radius: 0.8,
                        }),
                        Rc::clone(&rough_material),
                        sdf::ShapeOpType::SmoothUnion,
                        None,
                    );

                    let ground_node = scene.add_leaf_node(
                        Box::new(Cube {
                            center: Vector3f::new(0.0, -4.0, 0.0),
                            most_front_up_right: Vector3f::new(15.0, 0.25, 15.0),
                        }),
                        Rc::clone(&ground_material),
                    );

                    let mut prev_op: Option<&'_ ShapeOp<'_>> = None;
                    let n_objects = 16;
                    let n_colors = n_objects;
                    let saturation = Self::zigzag_factor(elpased_time, 2.0, 0.5, 0.8);
                    let value = Self::zigzag_factor(elpased_time, 2.0, 0.8, 0.95) as f64;
                    for i in 0..n_objects {
                        let hue = i as f64 / n_colors as f64;
                        let color = Self::hsv_to_rgb(hue, saturation.into(), value);
                        let emission_material =
                            Rc::new(PBRMaterial::new(color, color * 3.0, 0.0, 0.85, 0.05));
                        let fi = i as f64;
                        let time =
                            elpased_time as f64 * (f64::fract(fi * 412.531 + 0.513) - 0.5) * 2.0;
                        let mut center =
                            Vector3f::new(52.5126, 64.62744, 632.25) * fi + Vector3f::scalar(time);
                        center.x = f64::sin(center.x);
                        center.y = f64::sin(center.y);
                        center.z = f64::sin(center.z);
                        center = &center * &Vector3f::new(2.0, 2.0, 0.8);
                        let radius = lerp(0.3, 0.7, f64::fract(fi * 412.531 + 0.5124));
                        let current_op = scene.add_node(
                            Box::new(Sphere { center, radius }),
                            emission_material,
                            sdf::ShapeOpType::SmoothUnion,
                            prev_op,
                        );
                        prev_op = Some(current_op);
                    }
                    scene.add_root_node(ground_node);
                    scene.add_root_node(prev_op.unwrap());
                    scene.add_root_node(root_sphere_left);
                    scene.add_root_node(root_sphere_right);
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

    fn hsv_to_rgb(h: f64, s: f64, v: f64) -> Vector3f {
        let i = (h * 6.0).floor();
        let f = h * 6.0 - i;
        let p = v * (1.0 - s);
        let q = v * (1.0 - f * s);
        let t = v * (1.0 - (1.0 - f) * s);

        let (r, g, b) = match i as i32 % 6 {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            5 => (v, p, q),
            _ => (0.0, 0.0, 0.0), // Should not happen thanks to the modulo operator
        };
        Vector3f::new(r, g, b)
    }

    fn zigzag_factor(elapsed_time: f32, period: f32, min: f32, max: f32) -> f32 {
        let amplitude = max - min;
        let half_period = period / 2.0;
        let time = (elapsed_time % period) / half_period; // Normalizes time to a 0 to 2 range
        let zigzag = if time < 1.0 {
            time // Upward slope
        } else {
            2.0 - time // Downward slope
        };
        min + zigzag * amplitude
    }
}

impl Application for EmissionCubeApp {
    fn run(&self) -> Pin<Box<dyn Future<Output = ()>>> {
        Box::pin(async move {
            EmissionCubeApp::main().await;
        })
    }
}
