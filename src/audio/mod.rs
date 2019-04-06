extern crate cpal;

use parking_lot::Mutex;
use std::error::Error;
use std::sync::Arc;

use crate::samples::Samples;

pub mod cpal_backend;

#[cfg(feature = "pulse_simple_backend")]
pub mod pulse_simple_backend;

pub trait Backend {
    fn run(self);
}

pub struct BackendBuilder {
    pub capture_buf: Arc<Mutex<Samples>>,
    pub playback_buf: Arc<Mutex<Samples>>,
}

pub trait BackendBuilderFor<T: Backend> {
    fn build(self) -> Result<T, Box<Error>>;
}
