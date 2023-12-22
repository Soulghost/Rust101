use example::{emission_cubes::EmissionCubeApp, Application};

pub mod domain;
pub mod example;
pub mod material;
pub mod math;
pub mod node;
pub mod pipeline;
pub mod sdf;

pub async fn run() {}

fn main() {
    env_logger::init();
    run_application(Box::new(EmissionCubeApp {}));
}

fn run_application(app: Box<dyn Application>) {
    pollster::block_on(app.run());
}
