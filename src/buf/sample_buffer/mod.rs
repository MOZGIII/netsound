use crate::io::{ReadSamples, SamplesAvailable, WriteSamples};
use sample::Sample;

pub trait SampleBuffer:
    ReadSamples<<Self as SampleBuffer>::Sample>
    + WriteSamples<<Self as SampleBuffer>::Sample>
    + SamplesAvailable
{
    type Sample: Sample;
}

// mod from_frame_buffer;
mod vecdeque;

// pub use from_frame_buffer::*;
pub use vecdeque::*;
