use crate::samples::SharedSamples;
use std::error::Error;

pub mod cpal_backend;

#[cfg(feature = "pulse_simple_backend")]
pub mod pulse_simple_backend;

#[derive(Debug, Clone, Copy)]
pub struct Format {
    pub channels: u16,
    pub sample_rate: u32,
}

pub trait Backend: Send + Sync {
    fn run(&mut self);

    fn capture_format(&self) -> Format;
    fn playback_format(&self) -> Format;
}

#[derive(Debug)]
pub struct BackendBuilder<'a> {
    pub capture_buf: SharedSamples,
    pub playback_buf: SharedSamples,

    pub request_capture_formats: &'a [Format],
    pub request_playback_formats: &'a [Format],
}

pub trait BackendBuilderFor<T: Backend>: Sized {
    fn build(self) -> Result<T, Box<dyn Error>>;
}

pub trait BoxedBackendBuilderFor<'a, T: Backend + 'a> {
    type BackendType;
    fn build_boxed(self) -> Result<Box<dyn Backend + 'a>, Box<dyn Error>>;
}

impl<'a, TBackend, TBuilder> BoxedBackendBuilderFor<'a, TBackend> for TBuilder
where
    TBackend: Backend + 'a,
    TBuilder: BackendBuilderFor<TBackend>,
{
    type BackendType = TBackend;

    fn build_boxed(self) -> Result<Box<dyn Backend + 'a>, Box<dyn Error>> {
        Ok(Box::new(self.build()?))
    }
}
