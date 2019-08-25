mod decoder;
mod encoder;
mod error;

pub use self::decoder::Decoder;
pub use self::encoder::Encoder;
pub use self::error::Error;

use crate::audio::Format;
use audiopus::TryFrom;

pub fn make_encoder(format: Format, buf: Box<[f32]>) -> Result<Encoder, Error> {
    #[allow(unstable_name_collisions)]
    let sample_rate = audiopus::SampleRate::try_from(format.sample_rate as i32)?;
    let channels = audiopus::Channels::try_from(i32::from(format.channels))?;
    let enc = audiopus::coder::Encoder::new(sample_rate, channels, audiopus::Application::Audio)?;
    Ok(Encoder { opus: enc, buf })
}

pub fn make_decoder(format: Format, buf: Box<[f32]>) -> Result<Decoder, Error> {
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
pub enum FrameDurationMS {
    F2p5,
    F5,
    F10,
    F20,
    F40,
    F60,
}

#[allow(unused_variables)]
pub fn buf_size(format: Format, frame_duration_ms: FrameDurationMS, fec: bool) -> usize {
    // TODO: implement more optimized way of doing it.
    // See https://tools.ietf.org/html/rfc6716#section-2
    (format.sample_rate as usize) / 25 * 3
}
