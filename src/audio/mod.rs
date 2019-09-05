use crate::format::Format;
use sample::Sample;

mod builder;
pub use builder::*;

pub mod cpal_backend;

#[cfg(feature = "pulse_simple_backend")]
pub mod pulse_simple_backend;

pub trait Backend: Send + Sync {
    type CaptureSample: Sample;
    type PlaybackSample: Sample;

    fn run(&mut self);

    fn capture_format(&self) -> Format<Self::CaptureSample>;
    fn playback_format(&self) -> Format<Self::PlaybackSample>;
}
