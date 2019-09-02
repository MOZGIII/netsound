use sample::Sample;
use std::marker::PhantomData;

pub trait CpalInputConverter {
    type Sample: Sample;
    fn convert<'a>(&self, buf: &'a cpal::UnknownTypeInputBuffer<'a>) -> &'a [Self::Sample];
}

fn unknown_buffer_type() -> ! {
    panic!("Unexpected buffer type")
}

pub struct ExactCpalInputConverter<S: Sample> {
    phantom: PhantomData<S>,
}

impl<S: Sample> ExactCpalInputConverter<S> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl CpalInputConverter for ExactCpalInputConverter<u16> {
    type Sample = u16;

    fn convert<'a>(&self, buf: &'a cpal::UnknownTypeInputBuffer<'a>) -> &'a [Self::Sample] {
        match buf {
            cpal::UnknownTypeInputBuffer::U16(buffer) => &*buffer,
            _ => unknown_buffer_type(),
        }
    }
}

impl CpalInputConverter for ExactCpalInputConverter<i16> {
    type Sample = i16;

    fn convert<'a>(&self, buf: &'a cpal::UnknownTypeInputBuffer<'a>) -> &'a [Self::Sample] {
        match buf {
            cpal::UnknownTypeInputBuffer::I16(buffer) => &*buffer,
            _ => unknown_buffer_type(),
        }
    }
}

impl CpalInputConverter for ExactCpalInputConverter<f32> {
    type Sample = f32;

    fn convert<'a>(&self, buf: &'a cpal::UnknownTypeInputBuffer<'a>) -> &'a [Self::Sample] {
        match buf {
            cpal::UnknownTypeInputBuffer::F32(buffer) => &*buffer,
            _ => unknown_buffer_type(),
        }
    }
}
