use crate::sample::Sample;
use std::marker::PhantomData;

#[derive(Debug, Clone, Copy)]
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
