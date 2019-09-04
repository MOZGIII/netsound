use sample::{Frame, Sample};
use std::io::Result;

pub trait WriteSamples<S: Sample> {
    fn write_samples(&mut self, samples: &[S]) -> Result<usize>;
}

pub trait ReadSamples<S: Sample> {
    fn read_samples(&mut self, samples: &mut [S]) -> Result<usize>;
}

pub trait SamplesAvailable {
    fn samples_available(&self) -> Result<usize>;
}

pub trait WriteFrames<F: Frame> {
    fn write_frames(&mut self, frames: &[F]) -> Result<usize>;
}

pub trait ReadFrames<F: Frame> {
    fn read_frames(&mut self, frames: &mut [F]) -> Result<usize>;
}

pub trait FramesAvailable {
    fn frames_available(&self) -> Result<usize>;
}