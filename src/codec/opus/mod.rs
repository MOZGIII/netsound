mod decoder;
mod encoder;
mod error;

pub use self::decoder::Decoder;
pub use self::encoder::Encoder;
pub use self::error::Error;

use crate::audio::Format;
use audiopus::TryFrom;

pub fn make_encoder<'a>(format: &Format, buf: &'a mut [f32]) -> Result<Encoder<'a>, Error> {
    #[allow(unstable_name_collisions)]
    let sample_rate = audiopus::SampleRate::try_from(format.sample_rate as i32)?;
    let channels = audiopus::Channels::try_from(format.channels as i32)?;
    let enc = audiopus::coder::Encoder::new(sample_rate, channels, audiopus::Application::Audio)?;
    Ok(Encoder {
        opus: enc,
        buf: buf.as_mut(),
    })
}

pub fn make_decoder<'a>(format: &Format, buf: &'a mut [f32]) -> Result<Decoder<'a>, Error> {
    #[allow(unstable_name_collisions)]
    let sample_rate = audiopus::SampleRate::try_from(format.sample_rate as i32)?;
    let channels = audiopus::Channels::try_from(format.channels as i32)?;
    let dec = audiopus::coder::Decoder::new(sample_rate, channels)?;
    Ok(Decoder {
        opus: dec,
        buf: buf.as_mut(),
        fec: false,
    })
}
