use netsound_core::pcm::Sample;

pub trait CompatibleSample: Sample + cpal::SizedSample {}

impl<T> CompatibleSample for T where T: Sample + cpal::SizedSample {}

pub mod stream_config {
    use std::convert::TryInto;

    use super::CompatibleSample;
    use netsound_core::pcm::StreamConfig;

    fn sample_formats_do_not_match() -> ! {
        panic!("sample formats do not match")
    }

    /// # Panics
    ///
    /// Panics when the sample rate or channels don't fit in [`usize`].
    #[must_use]
    pub fn from_cpal<S: CompatibleSample>(config: &cpal::StreamConfig) -> StreamConfig<S> {
        let sample_rate: usize = config.sample_rate.0.try_into().unwrap();
        StreamConfig::<S>::new(sample_rate.into(), config.channels.try_into().unwrap())
    }

    /// # Panics
    ///
    /// Panics when the sample rate does not fit in [`usize`].
    #[must_use]
    pub fn from_cpal_supported<S: CompatibleSample>(
        config: &cpal::SupportedStreamConfig,
    ) -> StreamConfig<S> {
        if <S as cpal::SizedSample>::FORMAT != config.sample_format() {
            sample_formats_do_not_match();
        }
        let sample_rate: usize = config.sample_rate().0.try_into().unwrap();
        StreamConfig::<S>::new(sample_rate.into(), config.channels().into())
    }

    /// # Panics
    ///
    /// Panics when the sample rate does not fit in [`u32`].
    #[must_use]
    pub fn to_cpal<S: CompatibleSample>(config: StreamConfig<S>) -> cpal::StreamConfig {
        cpal::StreamConfig {
            channels: config.channels().try_into().unwrap(),
            sample_rate: cpal::SampleRate(config.sample_rate().as_usize().try_into().unwrap()),
            buffer_size: cpal::BufferSize::Default, // TODO: implement support for configuring buffer types.
        }
    }
}
