use crate::buf::{FrameBuffer, SampleBuffer};
use crate::io::{
    FramesAvailable, ReadFrames, ReadSamples, SamplesAvailable, WriteFrames, WriteSamples,
};
use sample::{Frame, Sample};
use std::collections::VecDeque;
use std::io::Result;

#[derive(Debug, Clone)]
pub struct VecDequeBuffer<T>(VecDeque<T>);

impl<T> VecDequeBuffer<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self(VecDeque::with_capacity(capacity))
    }
}

impl<A> std::iter::FromIterator<A> for VecDequeBuffer<A> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = A>,
    {
        Self(VecDeque::from_iter(iter))
    }
}

impl<T> From<Vec<T>> for VecDequeBuffer<T> {
    fn from(vec: Vec<T>) -> Self {
        Self(VecDeque::from(vec))
    }
}

impl<T> Into<Vec<T>> for VecDequeBuffer<T> {
    fn into(self) -> Vec<T> {
        self.0.into()
    }
}

impl<T> std::ops::Deref for VecDequeBuffer<T> {
    type Target = VecDeque<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for VecDequeBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> VecDequeBuffer<T> {
    fn read_items(&mut self, items: &mut [T]) -> Result<usize> {
        let vd = &mut self.0;
        let mut filled: usize = 0;
        for item_slot in items.iter_mut() {
            match vd.pop_front() {
                None => break,
                Some(item) => {
                    *item_slot = item;
                    filled += 1;
                }
            }
        }
        Ok(filled)
    }

    fn write_items(&mut self, items: &[T]) -> Result<usize>
    where
        T: Copy,
    {
        let vd = &mut self.0;
        let mut filled: usize = 0;
        for item in items.iter() {
            vd.push_back(*item);
            filled += 1;
        }
        Ok(filled)
    }

    fn items_available(&self) -> Result<usize> {
        Ok(self.0.len())
    }
}

impl<S: Sample> ReadSamples<S> for VecDequeBuffer<S> {
    fn read_samples(&mut self, samples: &mut [S]) -> Result<usize> {
        self.read_items(samples)
    }
}

impl<S: Sample> WriteSamples<S> for VecDequeBuffer<S> {
    fn write_samples(&mut self, samples: &[S]) -> Result<usize> {
        self.write_items(samples)
    }
}

impl<S: Sample> SamplesAvailable for VecDequeBuffer<S> {
    fn samples_available(&self) -> Result<usize> {
        self.items_available()
    }
}

impl<S: Sample> SampleBuffer for VecDequeBuffer<S> {
    type Sample = S;
}

impl<F: Frame> ReadFrames<F> for VecDequeBuffer<F> {
    fn read_frames(&mut self, frames: &mut [F]) -> Result<usize> {
        self.read_items(frames)
    }
}

impl<F: Frame> WriteFrames<F> for VecDequeBuffer<F> {
    fn write_frames(&mut self, frames: &[F]) -> Result<usize> {
        self.write_items(frames)
    }
}

impl<F: Frame> FramesAvailable for VecDequeBuffer<F> {
    fn frames_available(&self) -> Result<usize> {
        self.items_available()
    }
}

impl<F: Frame> FrameBuffer for VecDequeBuffer<F> {
    type Frame = F;
}
