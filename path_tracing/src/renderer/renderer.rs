use crate::domain::domain::Ray;
use crate::math::Math;
use crate::math::vector::Vector3f;
use crate::renderer::texture::RenderTextureSetMode;
use crate::scene::scene::Scene;
use crate::renderer::framebuffer::FrameBuffer;
use crate::util::logutil::LogUtil;

pub struct Renderer {
    pub fbo: Option<FrameBuffer>,
    pub spp: u32
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer { 
            fbo: None,
            spp: 16
        }
    }

    pub fn render(&mut self, scene: &Scene) -> Result<(), &'static str> {
        if self.fbo.is_none() {
            return Err("FBO not set");
        }

        let rt = self.fbo.as_mut().unwrap().get_render_target();
        let scale = f32::tan(Math::radian(scene.fov * 0.5));
        let aspect = scene.width as f32 / scene.height as f32;
        let eye_pos = Vector3f::new(278.0, 273.0, -800.0);
        println!("[Renderer] render info {} x {}, aspect {}, spp {}", scene.width, scene.height, aspect, scene.sample_per_pixel);
        
        println!("[Renderer] rt size {} x {}", rt.get_width(), rt.get_height());
        for j in 0..scene.height {
            for i in 0..scene.width {
                let x = (2.0 * (i as f32 + 0.5) / scene.width as f32 - 1.0) * aspect * scale;
                let y = (1.0 - 2.0 * (j as f32 + 0.5) / scene.height as f32) * scale;
                let dir = Vector3f::new(-x, y, 1.0).normalize();
                let ray = Ray::new(&eye_pos, &dir, 0.0);
                for _ in 0..scene.sample_per_pixel {
                    let color = scene.cast_ray(&ray).unwrap_or_else(|err| {
                        panic!("scene cast error {}", err);
                    });
                    rt.set(i, j, color / (self.spp as f32), RenderTextureSetMode::Add);
                }
            }
            LogUtil::log_progress("casting rays", j as f32 / scene.height as f32);
        }
        LogUtil::log_progress("casting rays", 1.0);
        println!();
        Ok(())
    }
}