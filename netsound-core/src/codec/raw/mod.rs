use crate::io::{AsyncReadItems, AsyncWriteItems};
use async_trait::async_trait;

mod codec;

pub type Endian = byteorder::LittleEndian;

#[derive(Debug)]
pub struct Encoder;

#[async_trait]
impl<T> super::Encoder<f32, T> for Encoder
where
    T: AsyncReadItems<f32> + Send + Unpin,
{
    async fn encode(
        &mut self,
        input: &mut T,
        output: &mut [u8],
    ) -> Result<usize, super::error::Encoding> {
        Ok(codec::encode::<Endian, T>(input, output)
            .await
            .map_err(|err| super::error::Encoding::Other(err.into()))?)
    }
}

#[derive(Debug)]
pub struct Decoder;

#[async_trait]
impl<T> super::Decoder<f32, T> for Decoder
where
    T: AsyncWriteItems<f32> + Send + Unpin,
{
    async fn decode(
        &mut self,
        input: &[u8],
        output: &mut T,
    ) -> Result<usize, super::error::Decoding> {
        Ok(codec::decode::<Endian, T>(input, output)
            .await
            .map_err(|err| super::error::Decoding::Other(err.into()))?)
    }
}
