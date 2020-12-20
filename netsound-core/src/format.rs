use crate::sample::Sample;
use crate::sample_type_name::sample_type_name;
use std::marker::PhantomData;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Format<S: Sample> {
    pub channels: u16,
    pub sample_rate: u32,
    sample_type: PhantomData<S>,
}

impl<S: Sample> Format<S> {
    pub const fn new(channels: u16, sample_rate: u32) -> Self {
        Self {
            channels,
            sample_rate,
            sample_type: PhantomData,
        }
    }
}

impl<S: Sample> std::fmt::Debug for Format<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Format")
            .field("channels", &self.channels)
            .field("sample_rate", &self.sample_rate)
            .field("sample_type", &sample_type_name::<S>())
            .finish()
    }
}

impl<S: Sample> std::fmt::Display for Format<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}; {}] @ {}",
            sample_type_name::<S>(),
            &self.channels,
            &self.sample_rate,
        )
    }
}
