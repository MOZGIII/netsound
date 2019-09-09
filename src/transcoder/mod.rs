use crate::io::{ItemsAvailable, ReadItems, WriteItems};

pub mod resampler;

pub trait Transcode<TFrom, TTo>: WriteItems<TFrom> + ReadItems<TTo> + ItemsAvailable<TTo> {
    type Ok;
    type Error;

    fn transcode(&mut self) -> Result<Self::Ok, Self::Error>;
}
