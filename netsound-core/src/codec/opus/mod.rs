mod decoder;
mod encoder;
mod error;

pub use decoder::Decoder;
pub use encoder::Encoder;
pub use error::Error;

use crate::pcm;
use audiopus::TryFrom as AudiopusTryFrom;
use std::convert::{TryFrom, TryInto};

pub fn make_encoder(
    stream_config: pcm::StreamConfig<f32>,
    buf: Box<[f32]>,
) -> Result<Encoder, Error> {
    let (sample_rate, channels) = convert_params(stream_config)?;
    let enc = audiopus::coder::Encoder::new(sample_rate, channels, audiopus::Application::Audio)?;
    Ok(Encoder { opus: enc, buf })
}

pub fn make_decoder(
    stream_config: pcm::StreamConfig<f32>,
    buf: Box<[f32]>,
) -> Result<Decoder, Error> {
    let (sample_rate, channels) = convert_params(stream_config)?;
    let dec = audiopus::coder::Decoder::new(sample_rate, channels)?;
    Ok(Decoder {
        opus: dec,
        buf,
        fec: false,
        channels: (channels as usize),
    })
}

fn convert_params(
    stream_config: pcm::StreamConfig<f32>,
) -> Result<(audiopus::SampleRate, audiopus::Channels), Error> {
    let sample_rate: usize = stream_config.sample_rate().into();
    #[allow(unstable_name_collisions)]
    let sample_rate = AudiopusTryFrom::try_from(i32::try_from(sample_rate)?)?;
    let channels: i32 = stream_config.channels().try_into()?;
    let channels = AudiopusTryFrom::try_from(channels)?;
    Ok((sample_rate, channels))
}

#[derive(Debug)]
pub enum SupportedFrameSizeMS {
    F2p5,
    F5,
    F10,
    F20,
    F40,
    F60,
}

impl SupportedFrameSizeMS {
    #[must_use]
    pub fn to_u32(&self) -> u32 {
        match self {
            SupportedFrameSizeMS::F2p5 => unimplemented!(),
            SupportedFrameSizeMS::F5 => 5,
            SupportedFrameSizeMS::F10 => 10,
            SupportedFrameSizeMS::F20 => 20,
            SupportedFrameSizeMS::F40 => 40,
            SupportedFrameSizeMS::F60 => 60,
        }
    }
}

#[derive(Debug)]
pub enum SupportedSampleRate {
    SR8,
    SR12,
    SR16,
    SR24,
    SR48,
}

#[allow(unused_variables)]
#[allow(clippy::needless_pass_by_value)]
#[must_use]
pub fn buf_size(
    sample_rate_hz: pcm::SampleRate,
    channels: pcm::Channels,
    frame_size_ms: SupportedFrameSizeMS,
    fec: bool,
) -> usize {
    // See https://tools.ietf.org/html/rfc6716#section-2
    frame_size_per_channel(usize::from(sample_rate_hz) / 1000, frame_size_ms.to_u32()) * channels
}

/// Takes Sample Rate in kHz and Frame Size in ms and returns frame size per
/// channel in bytes.
#[must_use]
pub const fn frame_size_per_channel(sample_rate_khz: usize, frame_size_ms: u32) -> usize {
    (sample_rate_khz as usize) * (frame_size_ms as usize)
}

/// Opus codec allows up to 120 ms frames.
pub const MAX_FRAME_SIZE_MS: u32 = 120;

/// Maximum sample rate supportws by opus is 48 kHz.
pub const MAX_SAMPLE_RATE_KHZ: usize = 48;

/// Maximum sample count per channel is 48 kHz * maximum frame size in
/// milliseconds.
pub const MAX_FRAME_SIZE_PER_CHANNEL: usize =
    frame_size_per_channel(MAX_SAMPLE_RATE_KHZ, MAX_FRAME_SIZE_MS);

/// Maximum channels supported by opus.
pub const MAX_CHANNELS: u32 = 2;

/// Maximum sample count per frame is 48 kHz * maximum frame size in
/// milliseconds * maximum number of channels.
pub const MAX_FRAME_SIZE: usize = MAX_FRAME_SIZE_PER_CHANNEL * (MAX_CHANNELS as usize);
