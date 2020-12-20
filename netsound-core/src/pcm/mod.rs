//! Pulse-code modulation (PCM) related types.
//!
//! More specifically, we focus on linear pulse-code modulation (LPCM).

mod frame;
mod sample;
mod sample_rate;
mod stream_config;

pub use frame::Frame;
pub use sample::Sample;
pub use sample_rate::SampleRate;
pub use stream_config::StreamConfig;

/// The type alias used to specify the amount of channels (i.e. in a frame).
pub type Channels = usize;
