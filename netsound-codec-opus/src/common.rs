use audiopus::TryFrom as AudiopusTryFrom;
use netsound_core::pcm;
use std::convert::{TryFrom, TryInto};

use crate::error;

pub fn convert_params(
    stream_config: pcm::StreamConfig<f32>,
) -> Result<(audiopus::SampleRate, audiopus::Channels), error::Init> {
    let sample_rate: usize = stream_config.sample_rate().into();
    #[allow(unstable_name_collisions)]
    let sample_rate = AudiopusTryFrom::try_from(i32::try_from(sample_rate)?)?;
    let channels: i32 = stream_config.channels().try_into()?;
    let channels = AudiopusTryFrom::try_from(channels)?;
    Ok((sample_rate, channels))
}
