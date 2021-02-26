use crate::{common::convert_params, error};
use async_trait::async_trait;
use audiopus::coder::Encoder as OpusEncoder;
use netsound_core::io::{AsyncReadItems, AsyncReadItemsExt, WaitMode};
use netsound_core::log::trace;
use netsound_core::pcm;

/// Opus encoder.
#[derive(Debug)]
pub struct Encoder {
    pub(super) opus: OpusEncoder,
    pub(super) buf: Box<[f32]>,
}

impl Encoder {
    /// Create a new [`Encoder`] with the specified params.
    ///
    /// # Errors
    ///
    /// Fails if the parameters validation fails or underlying opus codec
    /// library returns an error.
    pub fn new(
        stream_config: pcm::StreamConfig<f32>,
        buf: Box<[f32]>,
    ) -> Result<Self, error::Init> {
        let (sample_rate, channels) = convert_params(stream_config)?;
        let enc =
            audiopus::coder::Encoder::new(sample_rate, channels, audiopus::Application::Audio)?;
        Ok(Self { opus: enc, buf })
    }

    async fn encode_float<T>(
        &mut self,
        input: &mut T,
        output: &mut [u8],
    ) -> Result<usize, error::Op>
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
impl<T> netsound_core::codec::Encoder<f32, T> for Encoder
where
    T: AsyncReadItems<f32> + Unpin + Send,
{
    async fn encode(
        &mut self,
        input: &mut T,
        output: &mut [u8],
    ) -> Result<usize, netsound_core::codec::error::Encoding> {
        self.encode_float(input, output)
            .await
            .map_err(|err| err.into())
    }
}
