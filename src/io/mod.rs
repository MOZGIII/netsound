use sample::Sample;
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
