use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use crate::domain::domain::Ray;
use crate::math::Math;
use crate::math::vector::Vector3f;
use crate::renderer::texture::RenderTextureSetMode;
use crate::scene::scene::Scene;
use crate::renderer::framebuffer::FrameBuffer;
use crate::util::logutil::LogUtil;

pub struct Renderer {
    pub fbo: Option<Arc<Mutex<FrameBuffer>>>
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer { 
            fbo: None
        }
    }

    pub fn render(&mut self, scene: Arc<Scene>) -> Result<(), &'static str> {
        if self.fbo.is_none() {
            return Err("FBO not set");
        }

        let scale = f64::tan(Math::radian(scene.fov * 0.5));
        let aspect = scene.width as f64 / scene.height as f64;
        let eye_pos = Vector3f::new(278.0, 273.0, -800.0);
        {
            let fbo_mutex = Arc::clone(self.fbo.as_ref().unwrap());
            let mut fbo = fbo_mutex.lock().unwrap();
            let rt = fbo.get_render_target();
            println!("[Renderer] rt size {} x {}", rt.get_width(), rt.get_height());
        }
        
        let s = Arc::clone(&scene);
        let fbo_mutex = Arc::clone(self.fbo.as_ref().unwrap());
        let t = thread::spawn(move || {
            for j in 0..s.height {
                for i in 0..s.width {
                    let x = (2.0 * (i as f64 + 0.5) / s.width as f64 - 1.0) * aspect * scale;
                    let y = (1.0 - 2.0 * (j as f64 + 0.5) / s.height as f64) * scale;
                    let dir = Vector3f::new(-x, y, 1.0).normalize();
                    let ray = Ray::new(&eye_pos, &dir, 0.0);
                    for _ in 0..s.sample_per_pixel {
                        let (color, _) = s.cast_ray(&ray).unwrap_or_else(|err| {
                            panic!("scene cast error {}", err);
                        });
                        let mut fbo = fbo_mutex.lock().unwrap();
                        let rt = fbo.get_render_target();
                        rt.set(i, j, color / s.sample_per_pixel, RenderTextureSetMode::Add);
                    }
                }
                LogUtil::log_progress("casting rays", j as f32 / scene.height as f32);
            }
        });
        t.join().unwrap();
        LogUtil::log_progress("casting rays", 1.0);
        println!();
        Scene::print_stat();
        Ok(())
    }
}