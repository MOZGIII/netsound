use crate::io::{ItemsAvailable, ReadItems, WriteItems};
use sample::{Frame, Sample};

mod vecdeque;
pub use vecdeque::*;

pub trait Buffer:
    ReadItems<<Self as Buffer>::Item>
    + WriteItems<<Self as Buffer>::Item>
    + ItemsAvailable<<Self as Buffer>::Item>
{
    type Item;
}

pub trait SampleBuffer: Buffer<Item = <Self as SampleBuffer>::Sample> {
    type Sample: Sample;
}

pub trait FrameBuffer: Buffer<Item = <Self as FrameBuffer>::Frame> {
    type Frame: Frame;
}
