extern crate cpal;

use crate::samples::SharedSamples;
use std::error::Error;

pub mod cpal_backend;

#[cfg(feature = "pulse_simple_backend")]
pub mod pulse_simple_backend;

pub trait Backend {
    fn run(self);
}

pub struct BackendBuilder {
    pub capture_buf: SharedSamples,
    pub playback_buf: SharedSamples,
}

pub trait BackendBuilderFor<T: Backend> {
    fn build(self) -> Result<T, Box<Error>>;
}
