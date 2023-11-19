use crate::domain::Ray;
use crate::math::Math;
use crate::renderer::texture::RenderTextureSetMode;
use crate::sdf::Scene;
use crate::{math::Vector3f, renderer::framebuffer::FrameBuffer};
use indicatif::{ProgressBar, ProgressStyle};
use nalgebra::{Rotation3, Vector3};

pub struct Renderer {
    pub fbo: Option<FrameBuffer>,
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer { fbo: None }
    }

    pub fn render<'a>(
        &mut self,
        eye: Vector3f,
        rotation_degrees: Vector3f,
        scene: &'a Scene<'a>,
        silent: bool,
    ) -> Result<(), &'static str> {
        if self.fbo.is_none() {
            return Err("FBO not set");
        }

        let scale = f64::tan(Math::radian(scene.fov * 0.5));
        let aspect = scene.width as f64 / scene.height as f64;
        let eye_pos = eye;
        let fbo = self.fbo.as_mut().unwrap();
        let rt = fbo.get_render_target();
        let work_items: Vec<_> = (0..scene.height)
            .flat_map(|y| (0..scene.width).map(move |x| (x, y)))
            .collect();

        let m: Option<ProgressBar>;
        if !silent {
            println!(
                "[Renderer] rt size {} x {}, spp {}",
                rt.get_width(),
                rt.get_height(),
                scene.sample_per_pixel
            );

            let m_style = ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
            .progress_chars("##-");
            m = Some(ProgressBar::new(work_items.len() as _).with_style(m_style));
            m.as_ref().unwrap().println("[Renderer] ray marching...");
        } else {
            m = None;
        }

        work_items.iter().for_each(|point| {
            let (i, j) = *point;

            let x = (2.0 * (i as f64 + 0.5) / scene.width as f64 - 1.0) * aspect * scale;
            let y = (1.0 - 2.0 * (j as f64 + 0.5) / scene.height as f64) * scale;
            let mut dir = Vector3f::new(x, y, 1.0).normalize();
            // try to rotate the ray
            {
                let dir_a = Vector3::new(dir.x, dir.y, dir.z);
                let rotation = Rotation3::from_euler_angles(
                    rotation_degrees.x.to_radians(),
                    rotation_degrees.z.to_radians(),
                    rotation_degrees.y.to_radians(),
                );
                let dir_a: nalgebra::Matrix<
                    f64,
                    nalgebra::Const<3>,
                    nalgebra::Const<1>,
                    nalgebra::ArrayStorage<f64, 3, 1>,
                > = (rotation * dir_a).normalize();
                dir.x = dir_a.x;
                dir.y = dir_a.y;
                dir.z = dir_a.z;
            }
            let ray = Ray::new(&eye_pos, &dir, 0.0);
            let mut color = Vector3f::zero();
            for _ in 0..scene.sample_per_pixel {
                let sample_color = scene.cast_ray(&ray);
                color += sample_color / scene.sample_per_pixel;
            }
            rt.set(i, j, color, RenderTextureSetMode::Add);
            if !silent {
                m.as_ref().unwrap().inc(1);
            }
        });
        Ok(())
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
