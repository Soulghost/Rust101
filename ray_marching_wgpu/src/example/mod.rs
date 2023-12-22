use std::{future::Future, pin::Pin};

pub mod emission_cubes;

pub trait Application {
    fn run(&self) -> Pin<Box<dyn Future<Output = ()>>>;
}
