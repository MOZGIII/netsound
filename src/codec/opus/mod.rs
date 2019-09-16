mod decoder;
mod encoder;
mod error;

pub use self::decoder::Decoder;
pub use self::encoder::Encoder;
pub use self::error::Error;

use crate::format::Format;
use audiopus::TryFrom;

pub fn make_encoder(format: Format<f32>, buf: Box<[f32]>) -> Result<Encoder, Error> {
    #[allow(unstable_name_collisions)]
    let sample_rate = audiopus::SampleRate::try_from(format.sample_rate as i32)?;
    let channels = audiopus::Channels::try_from(i32::from(format.channels))?;
    let enc = audiopus::coder::Encoder::new(sample_rate, channels, audiopus::Application::Audio)?;
    Ok(Encoder { opus: enc, buf })
}

pub fn make_decoder(format: Format<f32>, buf: Box<[f32]>) -> Result<Decoder, Error> {
    #[allow(unstable_name_collisions)]
    let sample_rate = audiopus::SampleRate::try_from(format.sample_rate as i32)?;
    let channels = audiopus::Channels::try_from(i32::from(format.channels))?;
    let dec = audiopus::coder::Decoder::new(sample_rate, channels)?;
    Ok(Decoder {
        opus: dec,
        buf,
        fec: false,
        channels: (channels as usize),
    })
}

#[allow(dead_code)]
pub enum SupportedFrameSizeMS {
    F2p5,
    F5,
    F10,
    F20,
    F40,
    F60,
}

#[allow(dead_code)]
pub enum SupportedSampleRate {
    SR8,
    SR12,
    SR16,
    SR24,
    SR48,
}

#[allow(unused_variables)]
pub fn buf_size(
    sample_rate_khz: u32,
    channels: u16,
    frame_size_ms: SupportedFrameSizeMS,
    fec: bool,
) -> usize {
    // TODO: use smaller buffer size when possible.
    // See https://tools.ietf.org/html/rfc6716#section-2
    MAX_FRAME_SIZE_PER_CHANNEL as usize * channels as usize
}

/// Takes Sample Rate in kHz and Frame Size in ms and returns frame size per
/// channel in bytes.
#[allow(dead_code)]
pub const fn frame_size_per_channel(sample_rate_khz: u32, frame_size_ms: u32) -> usize {
    (sample_rate_khz as usize) * (frame_size_ms as usize)
}

/// Opus codec allows up to 120 ms frames.
#[allow(dead_code)]
pub const MAX_FRAME_SIZE_MS: u32 = 120;

/// Maximum sample rate supportws by opus is 48 kHz.
#[allow(dead_code)]
pub const MAX_SAMPLE_RATE_KHZ: u32 = 48;

/// Maximum sample count per channel is 48 kHz * maximum frame size in
/// milliseconds.
#[allow(dead_code)]
pub const MAX_FRAME_SIZE_PER_CHANNEL: usize =
    frame_size_per_channel(MAX_SAMPLE_RATE_KHZ, MAX_FRAME_SIZE_MS);

/// Maximum channels supported by opus.
#[allow(dead_code)]
pub const MAX_CHANNELS: u32 = 2;

/// Maximum sample count per frame is 48 kHz * maximum frame size in
/// milliseconds * maximum number of channels.
#[allow(dead_code)]
pub const MAX_FRAME_SIZE: usize = MAX_FRAME_SIZE_PER_CHANNEL * (MAX_CHANNELS as usize);
