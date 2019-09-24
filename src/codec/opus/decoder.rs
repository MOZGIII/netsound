use super::Error;
use crate::io::{AsyncWriteItems, AsyncWriteItemsExt};
use async_trait::async_trait;
use audiopus::coder::Decoder as OpusDecoder;

#[derive(Debug)]
pub struct Decoder {
    pub(super) opus: OpusDecoder,
    pub(super) buf: Box<[f32]>,
    pub(super) fec: bool,
    pub(super) channels: usize,
}

// TODO: switch to `opus` crate and remove this.
unsafe impl Send for Decoder {}

impl Decoder {
    pub async fn decode_float<T>(
        &mut self,
        input: &[u8],
        output: &mut T,
        fec: bool,
    ) -> Result<usize, Error>
    where
        T: AsyncWriteItems<f32> + Unpin,
    {
        let audiosize = {
            let buf = &mut self.buf[..];
            self.opus.decode_float(input, buf, fec)?
        };
        let bufsize = audiosize * self.channels;
        let size = output.write_items(&self.buf[..bufsize]).await?;
        Ok(size)
    }
}

#[async_trait]
impl<T> super::super::Decoder<f32, T> for Decoder
where
    T: AsyncWriteItems<f32> + Unpin + Send,
{
    async fn decode(
        &mut self,
        input: &[u8],
        output: &mut T,
    ) -> Result<usize, super::super::DecodingError> {
        self.decode_float(input, output, self.fec)
            .await
            .map_err(|err| err.into())
    }
}
