use crate::io::{AsyncItemsAvailable, AsyncReadItems, AsyncWriteItems};
use async_trait::async_trait;

pub mod noop;
pub mod resampler;

#[async_trait]
pub trait Transcode {
    type Ok;
    type Error;

    async fn transcode(&mut self) -> Result<Self::Ok, Self::Error>;
}
