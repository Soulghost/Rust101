use std::sync::{mpsc, Arc};

use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use crate::domain::domain::Ray;
use crate::math::vector::Vector3f;
use crate::math::Math;
use crate::renderer::framebuffer::FrameBuffer;
use crate::renderer::texture::RenderTextureSetMode;
use crate::scene::scene::Scene;

pub struct Renderer {
    pub fbo: Option<FrameBuffer>,
}

struct RenderMessage {
    pub x: u32,
    pub y: u32,
    pub color: Vector3f,
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer { fbo: None }
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
        println!(
            "[Renderer] rt size {} x {}, spp {}",
            rt.get_width(),
            rt.get_height(),
            scene.sample_per_pixel
        );

        let work_items: Vec<_> = (0..scene.height)
            .flat_map(|y| (0..scene.width).map(move |x| ((x, y))))
            .collect();

        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(n_threads as usize + 1) // 1 extra thread for reducing
            .build()
            .unwrap();
        pool.scope(|s| {
            let (tx, rx) = mpsc::channel::<RenderMessage>();

            s.spawn(|_| {
                // progress bar
                let m_style = ProgressStyle::with_template(
                    "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
                )
                .unwrap()
                .progress_chars("##-");
                let m = ProgressBar::new(work_items.len() as _).with_style(m_style);

                m.println(format!("ray tracing using {n_threads} threads..."));

                for received in rx {
                    rt.set(
                        received.x,
                        received.y,
                        received.color,
                        RenderTextureSetMode::Add,
                    );
                    m.inc(1);
                }
            });

            work_items.par_iter().for_each(|point| {
                let (i, j) = *point;

                let x = (2.0 * (i as f64 + 0.5) / scene.width as f64 - 1.0) * aspect * scale;
                let y = (1.0 - 2.0 * (j as f64 + 0.5) / scene.height as f64) * scale;
                let dir = Vector3f::new(-x, y, 1.0).normalize();
                let ray = Ray::new(&eye_pos, &dir, 0.0);
                let mut color = Vector3f::zero();
                for _ in 0..scene.sample_per_pixel {
                    let (sample_color, _) = scene.cast_ray(&ray).unwrap_or_else(|err| {
                        panic!("scene cast error {}", err);
                    });
                    color += sample_color / scene.sample_per_pixel;
                }
                tx.send(RenderMessage { x: i, y: j, color })
                    .expect("renderer message send failure");
            });
        });
        Ok(())
    }
}
