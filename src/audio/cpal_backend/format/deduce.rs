use sample::Sample;
use std::marker::PhantomData;

pub trait CpalSampleFormatDeducer {
    type Sample: Sample;
    fn deduce(&self) -> cpal::SampleFormat;
}

pub struct ExactCpalSampleFormatDeducer<S: Sample> {
    phantom: PhantomData<S>,
}

impl<S: Sample> ExactCpalSampleFormatDeducer<S> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl CpalSampleFormatDeducer for ExactCpalSampleFormatDeducer<u16> {
    type Sample = u16;

    fn deduce(&self) -> cpal::SampleFormat {
        cpal::SampleFormat::U16
    }
}

impl CpalSampleFormatDeducer for ExactCpalSampleFormatDeducer<i16> {
    type Sample = i16;

    fn deduce(&self) -> cpal::SampleFormat {
        cpal::SampleFormat::I16
    }
}

impl CpalSampleFormatDeducer for ExactCpalSampleFormatDeducer<f32> {
    type Sample = f32;

    fn deduce(&self) -> cpal::SampleFormat {
        cpal::SampleFormat::F32
    }
}
