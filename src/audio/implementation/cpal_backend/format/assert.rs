use sample::Sample;
use std::marker::PhantomData;

pub trait CpalSampleFormatAsserter {
    type Sample: Sample;
    fn assert(&self, sample_format: cpal::SampleFormat);
}

fn unexpected_format_type() -> ! {
    panic!("Unexpected format type")
}

#[allow(dead_code)]
pub struct ExactCpalSampleFormatAsserter<S: Sample> {
    phantom: PhantomData<S>,
}

#[allow(dead_code)]
impl<S: Sample> ExactCpalSampleFormatAsserter<S> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl CpalSampleFormatAsserter for ExactCpalSampleFormatAsserter<u16> {
    type Sample = u16;

    fn assert(&self, sample_format: cpal::SampleFormat) {
        if sample_format != cpal::SampleFormat::U16 {
            unexpected_format_type()
        }
    }
}

impl CpalSampleFormatAsserter for ExactCpalSampleFormatAsserter<i16> {
    type Sample = i16;

    fn assert(&self, sample_format: cpal::SampleFormat) {
        if sample_format != cpal::SampleFormat::I16 {
            unexpected_format_type()
        }
    }
}

impl CpalSampleFormatAsserter for ExactCpalSampleFormatAsserter<f32> {
    type Sample = f32;

    fn assert(&self, sample_format: cpal::SampleFormat) {
        if sample_format != cpal::SampleFormat::F32 {
            unexpected_format_type()
        }
    }
}
