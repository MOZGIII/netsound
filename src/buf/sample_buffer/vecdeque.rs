use super::SampleBuffer;
use crate::io::{ReadSamples, SamplesAvailable, WriteSamples};
use sample::Sample;
use std::collections::VecDeque;
use std::io::Result;

#[derive(Debug, Clone)]
pub struct VecDequeSampleBuffer<S: Sample>(VecDeque<S>);

impl<S: Sample> VecDequeSampleBuffer<S> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self(VecDeque::with_capacity(capacity))
    }
}

impl<S: Sample> std::iter::FromIterator<S> for VecDequeSampleBuffer<S> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = S>,
    {
        Self(VecDeque::from_iter(iter))
    }
}

impl<S: Sample> From<Vec<S>> for VecDequeSampleBuffer<S> {
    fn from(vec: Vec<S>) -> Self {
        Self(VecDeque::from(vec))
    }
}

impl<S: Sample> Into<Vec<S>> for VecDequeSampleBuffer<S> {
    fn into(self) -> Vec<S> {
        self.0.into()
    }
}

impl<S: Sample> std::ops::Deref for VecDequeSampleBuffer<S> {
    type Target = VecDeque<S>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S: Sample> std::ops::DerefMut for VecDequeSampleBuffer<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<S: Sample> ReadSamples<S> for VecDequeSampleBuffer<S> {
    fn read_samples(&mut self, samples: &mut [S]) -> Result<usize> {
        let vd = &mut self.0;
        let mut filled: usize = 0;
        for sample_slot in samples.iter_mut() {
            match vd.pop_front() {
                None => break,
                Some(sample) => {
                    *sample_slot = sample;
                    filled += 1;
                }
            }
        }
        Ok(filled)
    }
}

impl<S: Sample> WriteSamples<S> for VecDequeSampleBuffer<S> {
    fn write_samples(&mut self, samples: &[S]) -> Result<usize> {
        let vd = &mut self.0;
        let mut filled: usize = 0;
        for sample in samples.iter() {
            vd.push_back(*sample);
            filled += 1;
        }
        Ok(filled)
    }
}

impl<S: Sample> SamplesAvailable for VecDequeSampleBuffer<S> {
    fn samples_available(&self) -> Result<usize> {
        Ok(self.0.len())
    }
}

impl<S: Sample> SampleBuffer for VecDequeSampleBuffer<S> {
    type Sample = S;
}
