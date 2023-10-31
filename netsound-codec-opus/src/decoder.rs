use crate::{common::convert_params, error};
use async_trait::async_trait;
use audiopus::coder::Decoder as OpusDecoder;
use netsound_core::io::{AsyncWriteItems, AsyncWriteItemsExt, WaitMode};
use netsound_core::log::trace;
use netsound_core::pcm;

/// Opus decoder.
#[derive(Debug)]
pub struct Decoder {
    pub(super) opus: OpusDecoder,
    pub(super) buf: Box<[f32]>,
    pub(super) fec: bool,
    pub(super) channels: usize,
}

impl Decoder {
    /// Create a new [`Decoder`] with the specified params.
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
        let dec = audiopus::coder::Decoder::new(sample_rate, channels)?;
        Ok(Self {
            opus: dec,
            buf,
            fec: false,
            channels: (channels as usize),
        })
    }

    async fn decode_float<T>(
        &mut self,
        input: &[u8],
        output: &mut T,
        fec: bool,
    ) -> Result<usize, error::Op>
    where
        T: AsyncWriteItems<f32> + Unpin,
    {
        let audiosize = {
            let buf = &mut self.buf[..];
            trace!("opus: decoding buf {}", buf.len());
            self.opus.decode_float(Some(input), buf, fec)?
        };
        let bufsize = audiosize * self.channels;
        let size = output
            .write_items(&self.buf[..bufsize], WaitMode::WaitForReady)
            .await?;
        Ok(size)
    }
}

#[async_trait]
impl<T> netsound_core::codec::Decoder<f32, T> for Decoder
where
    T: AsyncWriteItems<f32> + Unpin + Send,
{
    async fn decode(
        &mut self,
        input: &[u8],
        output: &mut T,
    ) -> Result<usize, netsound_core::codec::error::Decoding> {
        self.decode_float(input, output, self.fec)
            .await
            .map_err(Into::into)
    }
}
