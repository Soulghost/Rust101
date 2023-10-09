extern crate lazy_static;

use std::sync::Arc;
use material::material::LitMaterial;
use math::vector::Vector3f;
use mesh::model::Model;
use scene::scene::Scene;

use crate::renderer::{renderer::Renderer, framebuffer::FrameBuffer};

pub mod scene;
pub mod mesh;
pub mod bvh;
pub mod math;
pub mod domain;
pub mod material;
pub mod renderer;
pub mod util;

fn main() {
    let width = 200; // 784
    let height = 200; // 784
    let spp = 4; // 16
    let mut scene = Scene::new(width, 
                                      height, 
                                  40.0, 
              Vector3f::new(0.235294, 0.67451, 0.843137), 
                     0.8,
                     spp);

    let white_mat = Arc::new(
        LitMaterial::new(&Vector3f::new(0.725, 0.71, 0.68), &Vector3f::zero())
    );
    let red_mat = Arc::new(
        LitMaterial::new(&Vector3f::new(0.63, 0.065, 0.05), &Vector3f::zero())
    );
    let green_mat = Arc::new(
        LitMaterial::new(&Vector3f::new(0.14, 0.45, 0.091), &Vector3f::zero())
    );

    let light_emission_color = Vector3f::new(0.747 + 0.058, 0.747 + 0.258, 0.747) * 8.0 + Vector3f::new(0.740 + 0.287, 0.740 + 0.160, 0.740) * 15.6 + Vector3f::new(0.737 + 0.642, 0.737 + 0.159,0.737) * 18.4;
    let light_color = Vector3f::new(0.65, 0.65, 0.65);
    let light_mat = Arc::new(
        LitMaterial::new(&light_color,
        &light_emission_color)
    );
    let floor = Arc::new(
        Model::new("./resource/cornellbox/floor.obj", white_mat.clone())
    );
    let shortbox = Arc::new(
        Model::new("./resource/cornellbox/shortbox.obj", white_mat.clone())
    );
    let tallbox = Arc::new(
        Model::new("./resource/cornellbox/tallbox.obj", white_mat.clone())
    );
    let left = Arc::new(
        Model::new("./resource/cornellbox/left.obj", red_mat.clone())
    );
    let right = Arc::new(
        Model::new("./resource/cornellbox/right.obj", green_mat.clone())
    );
    let light = Arc::new(
        Model::new("./resource/cornellbox/light.obj", light_mat.clone())
    );
      
    scene.add(floor);
    scene.add(shortbox);
    scene.add(tallbox);
    scene.add(left);
    scene.add(right);
    scene.add(light);

    scene.build_bvh();

    let mut renderer = Renderer::new();
    let fbo = FrameBuffer::new(width, height);
    renderer.fbo = Some(fbo);

    println!("[Main] start rendering...");
    renderer.render(&scene).unwrap_or_else(|err| {
        panic!("[Main] renderer error {}", err);
    });
    println!("[Main] end rendering...");

    renderer.fbo.as_mut().unwrap().get_render_target().dump_to_file("out/result.ppm").unwrap_or_else(|err| {
        panic!("[Main] dump rt to file error {}", err);
    });
}
