mod builder;
pub use builder::*;

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
