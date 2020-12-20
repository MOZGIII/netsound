use super::pcm::{SampleRate, StreamConfig};

static STREAM_CONFIGS: &[StreamConfig<f32>] = &[
    StreamConfig::new(SampleRate::from_usize(48000), 2),
    StreamConfig::new(SampleRate::from_usize(48000), 1),
    StreamConfig::new(SampleRate::from_usize(24000), 2),
    StreamConfig::new(SampleRate::from_usize(24000), 1),
    StreamConfig::new(SampleRate::from_usize(16000), 2),
    StreamConfig::new(SampleRate::from_usize(16000), 1),
    StreamConfig::new(SampleRate::from_usize(12000), 2),
    StreamConfig::new(SampleRate::from_usize(12000), 1),
    StreamConfig::new(SampleRate::from_usize(8000), 2),
    StreamConfig::new(SampleRate::from_usize(8000), 1),
];

pub fn input() -> &'static [StreamConfig<f32>] {
    STREAM_CONFIGS
}

pub fn output() -> &'static [StreamConfig<f32>] {
    STREAM_CONFIGS
}
