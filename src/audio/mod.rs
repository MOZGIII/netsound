extern crate cpal;

use crate::samples::SharedSamples;
use std::error::Error;

pub mod cpal_backend;

#[cfg(feature = "pulse_simple_backend")]
pub mod pulse_simple_backend;

pub trait Backend: Send + Sync {
    fn run(self);
}

pub struct BackendBuilder {
    pub capture_buf: SharedSamples,
    pub playback_buf: SharedSamples,
}

pub trait BackendBuilderFor<T: Backend>: Sized {
    fn build(self) -> Result<T, Box<Error>>;
}

pub trait BoxedBackendBuilderFor<'a, T: Backend + 'a>: BackendBuilderFor<T> {
    fn build_boxed(self) -> Result<Box<Backend + 'a>, Box<Error>> {
        Ok(Box::new(self.build()?))
    }
}

impl<'a, TBackend: Backend + 'a, TBuilder: BackendBuilderFor<TBackend>>
    BoxedBackendBuilderFor<'a, TBackend> for TBuilder
{
}
