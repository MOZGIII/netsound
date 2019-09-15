use super::*;
use sample::Sample;
use std::io::Result;
use std::marker::PhantomData;

/// Noop acts as writer, reader and transcoder.
#[allow(dead_code)]
#[derive(Debug)]
pub struct Noop<S: Sample, T: WriteItems<S> + ReadItems<S> + ItemsAvailable<S>> {
    buf: T,
    sample_type: PhantomData<S>,
}

#[allow(dead_code)]
impl<S: Sample, T: WriteItems<S> + ReadItems<S> + ItemsAvailable<S>> Noop<S, T> {
    pub fn new(buf: T) -> Self {
        Self {
            buf,
            sample_type: PhantomData,
        }
    }
}

impl<S: Sample, T: WriteItems<S> + ReadItems<S> + ItemsAvailable<S>> WriteItems<S> for Noop<S, T> {
    fn write_items(&mut self, items: &[S]) -> Result<usize> {
        self.buf.write_items(items)
    }
}

impl<S: Sample, T: WriteItems<S> + ReadItems<S> + ItemsAvailable<S>> ReadItems<S> for Noop<S, T> {
    fn read_items(&mut self, items: &mut [S]) -> Result<usize> {
        self.buf.read_items(items)
    }
}

impl<S: Sample, T: WriteItems<S> + ReadItems<S> + ItemsAvailable<S>> ItemsAvailable<S>
    for Noop<S, T>
{
    fn items_available(&self) -> Result<usize> {
        self.buf.items_available()
    }
}

impl<S: Sample, T: WriteItems<S> + ReadItems<S> + ItemsAvailable<S>> Transcode<S, S>
    for Noop<S, T>
{
    type Ok = ();
    type Error = ();

    fn transcode(&mut self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
