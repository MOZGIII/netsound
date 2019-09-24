use crate::io::{AsyncItemsAvailable, AsyncReadItems, AsyncWriteItems};

pub mod noop;
pub mod resampler;

pub trait Transcode<TFrom: Unpin, TTo: Unpin>:
    AsyncWriteItems<TFrom> + AsyncReadItems<TTo> + AsyncItemsAvailable<TTo>
{
    type Ok;
    type Error;

    fn transcode(&mut self) -> Result<Self::Ok, Self::Error>;
}
