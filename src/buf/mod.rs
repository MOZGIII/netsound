use crate::io::{AsyncItemsAvailable, AsyncReadItems, AsyncWriteItems};
use crate::sample::Sample;
use sample::Frame;

mod vecdeque;
pub use vecdeque::*;

pub trait Buffer:
    AsyncReadItems<<Self as Buffer>::Item>
    + AsyncWriteItems<<Self as Buffer>::Item>
    + AsyncItemsAvailable<<Self as Buffer>::Item>
    + Send
    + Unpin
{
    type Item: Unpin;
}

pub trait SampleBuffer: Buffer<Item = <Self as SampleBuffer>::Sample> {
    type Sample: Sample;
}

pub trait FrameBuffer: Buffer<Item = <Self as FrameBuffer>::Frame> {
    type Frame: Frame + Unpin;
}
