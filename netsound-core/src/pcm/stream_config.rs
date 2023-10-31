use super::{Channels, Frame, Sample, SampleRate};
use std::any::type_name;
use std::fmt;
use std::marker::PhantomData;

/// A PCM stream config.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct StreamConfig<S: Sample> {
    sample_rate: SampleRate,
    channels: Channels,
    data_type: PhantomData<S>,
}

impl<S: Sample> StreamConfig<S> {
    /// Create a new [`StreamConfig`].
    #[must_use]
    pub const fn new(sample_rate: SampleRate, channels: Channels) -> Self {
        Self {
            sample_rate,
            channels,
            data_type: PhantomData,
        }
    }

    /// Create a new [`StreamConfig`] from a [`Frame`].
    #[must_use]
    pub const fn from_frame<F>(sample_rate: SampleRate) -> Self
    where
        F: Frame<Sample = S>,
    {
        Self::new(sample_rate, <F as dasp_frame::Frame>::CHANNELS)
    }

    /// The sample rate.
    #[must_use]
    pub const fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    /// The amount of channels.
    #[must_use]
    pub const fn channels(&self) -> Channels {
        self.channels
    }

    /// Returns the rust name of the sample type.
    #[must_use]
    pub fn sample_type_name() -> &'static str {
        type_name::<S>()
    }
}

impl<S: Sample> fmt::Debug for StreamConfig<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StreamConfig")
            .field("sample_rate", &self.sample_rate())
            .field("channels", &self.channels())
            .field("sample_type", &Self::sample_type_name())
            .finish()
    }
}

impl<S: Sample> fmt::Display for StreamConfig<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}; {}] @ {}",
            Self::sample_type_name(),
            self.channels(),
            self.sample_rate(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        let stream_config = StreamConfig::<f32>::new(SampleRate::from_usize(48000), 2);
        assert_eq!(format!("{stream_config}"), "[f32; 2] @ 48000");
    }
}
