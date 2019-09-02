use sample::Sample;
use std::marker::PhantomData;

pub trait CpalOutputConverter {
    type Sample: Sample;
    fn convert<'a>(&self, buf: &'a mut cpal::UnknownTypeOutputBuffer<'a>)
        -> &'a mut [Self::Sample];
}

fn unknown_buffer_type() -> ! {
    panic!("Unexpected buffer type")
}

pub struct ExactCpalOutputConverter<S: Sample> {
    phantom: PhantomData<S>,
}

impl<S: Sample> ExactCpalOutputConverter<S> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl CpalOutputConverter for ExactCpalOutputConverter<u16> {
    type Sample = u16;

    fn convert<'a>(
        &self,
        buf: &'a mut cpal::UnknownTypeOutputBuffer<'a>,
    ) -> &'a mut [Self::Sample] {
        match buf {
            cpal::UnknownTypeOutputBuffer::U16(buffer) => &mut *buffer,
            _ => unknown_buffer_type(),
        }
    }
}

impl CpalOutputConverter for ExactCpalOutputConverter<i16> {
    type Sample = i16;

    fn convert<'a>(
        &self,
        buf: &'a mut cpal::UnknownTypeOutputBuffer<'a>,
    ) -> &'a mut [Self::Sample] {
        match buf {
            cpal::UnknownTypeOutputBuffer::I16(buffer) => &mut *buffer,
            _ => unknown_buffer_type(),
        }
    }
}

impl CpalOutputConverter for ExactCpalOutputConverter<f32> {
    type Sample = f32;

    fn convert<'a>(
        &self,
        buf: &'a mut cpal::UnknownTypeOutputBuffer<'a>,
    ) -> &'a mut [Self::Sample] {
        match buf {
            cpal::UnknownTypeOutputBuffer::F32(buffer) => &mut *buffer,
            _ => unknown_buffer_type(),
        }
    }
}
