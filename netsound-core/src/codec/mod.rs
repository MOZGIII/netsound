use crate::io::{AsyncReadItems, AsyncWriteItems};
use crate::pcm::Sample;
use async_trait::async_trait;

pub mod raw;

pub mod error;

#[async_trait]
pub trait Encoder<S: Sample, T: AsyncReadItems<S>> {
    async fn encode(&mut self, input: &mut T, output: &mut [u8]) -> Result<usize, error::Encoding>;
}

#[async_trait]
pub trait Decoder<S: Sample, T: AsyncWriteItems<S>> {
    async fn decode(&mut self, input: &[u8], output: &mut T) -> Result<usize, error::Decoding>;
}
