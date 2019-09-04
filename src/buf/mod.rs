use crate::io::{
    FramesAvailable, ReadFrames, ReadSamples, SamplesAvailable, WriteFrames, WriteSamples,
};
use sample::{Frame, Sample};

mod vecdeque;
pub use vecdeque::*;

pub trait SampleBuffer:
    ReadSamples<<Self as SampleBuffer>::Sample>
    + WriteSamples<<Self as SampleBuffer>::Sample>
    + SamplesAvailable
{
    type Sample: Sample;
}

pub trait FrameBuffer:
    ReadFrames<<Self as FrameBuffer>::Frame>
    + WriteFrames<<Self as FrameBuffer>::Frame>
    + FramesAvailable
{
    type Frame: Frame;
}
