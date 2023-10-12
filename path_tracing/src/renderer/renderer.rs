use std::sync::{Arc, mpsc};
use std::thread::{self, JoinHandle};

use indicatif::{MultiProgress, ProgressStyle, ProgressBar};

use crate::domain::domain::Ray;
use crate::math::Math;
use crate::math::vector::Vector3f;
use crate::renderer::texture::RenderTextureSetMode;
use crate::scene::scene::Scene;
use crate::renderer::framebuffer::FrameBuffer;

pub struct Renderer {
    pub fbo: Option<FrameBuffer>
}

struct RenderMessage {
    pub x: u32,
    pub y: u32,
    pub color: Vector3f
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer { 
            fbo: None
        }
    }

    pub fn render(&mut self, scene: Arc<Scene>, n_threads: u32) -> Result<(), &'static str> {
        if self.fbo.is_none() {
            return Err("FBO not set");
        }

        let scale = f64::tan(Math::radian(scene.fov * 0.5));
        let aspect = scene.width as f64 / scene.height as f64;
        let eye_pos = Vector3f::new(278.0, 273.0, -800.0);
        let fbo = self.fbo.as_mut().unwrap();
        let rt = fbo.get_render_target();
        println!("[Renderer] rt size {} x {}, spp {}", rt.get_width(), rt.get_height(), scene.sample_per_pixel);

        let mut thread_handles: Vec<JoinHandle<()>> = vec![];
        let mut thread_index = 0;

        // progress bar
        let m = MultiProgress::new();
        let m_style = ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
        .progress_chars("##-");


        let (tx, rx) = mpsc::channel::<RenderMessage>();
        m.println("ray tracing:").unwrap();
        let gap = scene.height / n_threads;
        for k in (0..scene.height).step_by(gap as usize) {
            let end = u32::min(k + gap, scene.height);
            let s = Arc::clone(&scene);
            let e = eye_pos.clone();
            let index = thread_index;
            thread_index += 1;
            let pb = m.add(ProgressBar::new(((end - k) * s.width) as u64));
            pb.set_message(format!("renderer #{}", index));
            pb.set_style(m_style.clone());

            let sender = tx.clone();
            let t = thread::spawn(move || {
                for j in k..end {
                    for i in 0..s.width {
                        let x = (2.0 * (i as f64 + 0.5) / s.width as f64 - 1.0) * aspect * scale;
                        let y = (1.0 - 2.0 * (j as f64 + 0.5) / s.height as f64) * scale;
                        let dir = Vector3f::new(-x, y, 1.0).normalize();
                        let ray = Ray::new(&e, &dir, 0.0);
                        for _ in 0..s.sample_per_pixel {
                            let (color, _) = s.cast_ray(&ray).unwrap_or_else(|err| {
                                panic!("scene cast error {}", err);
                            });
                            let color = color / s.sample_per_pixel;
                            sender.send(RenderMessage {
                                x: i,
                                y: j,
                                color
                            }).expect("renderer message send failure");
                        }
                    }
                    pb.inc(s.width as u64);
                }
                let msg = format!("renderer #{} finished", index);
                pb.finish_with_message(msg);
            });
            thread_handles.push(t);
        }
        drop(tx);
        
        for received in rx {
            rt.set(received.x, received.y, received.color, RenderTextureSetMode::Add)
        }

        println!();
        Ok(())
    }
}