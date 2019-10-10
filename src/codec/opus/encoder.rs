use super::Error;
use crate::io::{AsyncReadItems, AsyncReadItemsExt, WaitMode};
use crate::log::*;
use async_trait::async_trait;
use audiopus::coder::Encoder as OpusEncoder;

#[derive(Debug)]
pub struct Encoder {
    pub(super) opus: OpusEncoder,
    pub(super) buf: Box<[f32]>,
}

// TODO: switch to `opus` crate and remove this.
unsafe impl Send for Encoder {}

impl Encoder {
    pub async fn encode_float<T>(
        &mut self,
        input: &mut T,
        output: &mut [u8],
    ) -> Result<usize, Error>
    where
        T: AsyncReadItems<f32> + Unpin,
    {
        input
            .read_exact_items(&mut self.buf, WaitMode::WaitForReady)
            .await?;
        trace!("opus: encoding buf {}", self.buf.len());
        let bytes_written = self.opus.encode_float(&self.buf, output)?;
        Ok(bytes_written)
    }
}

#[async_trait]
impl<T> super::super::Encoder<f32, T> for Encoder
where
    T: AsyncReadItems<f32> + Unpin + Send,
{
    async fn encode(
        &mut self,
        input: &mut T,
        output: &mut [u8],
    ) -> Result<usize, super::super::EncodingError> {
        self.encode_float(input, output)
            .await
            .map_err(|err| err.into())
    }
}
