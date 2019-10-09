use crate::codec::{Encoder, EncodingError};
use crate::io::AsyncReadItems;
use crate::log::*;
use crate::sample::Sample;
use crate::transcode::Transcode;
use std::marker::PhantomData;
use std::net::SocketAddr;
use tokio::net::udp::split::UdpSocketSendHalf;

use super::*;

#[derive(Debug, Default)]
pub struct SendStats {
    pub frames_encoded: usize,
    pub bytes_encoded: usize,
    pub not_enough_data_at_encoding_errors: usize,
    pub packets_sent: usize,
    pub bytes_sent: usize,
    pub bytes_sent_mismatches: usize,
}

pub struct SendService<'a, TCaptureSample, TCaptureDataReader, TCaptureTranscoder, TEncoder>
where
    TCaptureSample: Sample,
    TCaptureDataReader: AsyncReadItems<TCaptureSample>,
    TCaptureTranscoder: Transcode,
    TEncoder: Encoder<TCaptureSample, TCaptureDataReader> + ?Sized,
{
    pub capture_sample: PhantomData<TCaptureSample>,
    pub capture_data_reader: TCaptureDataReader,
    pub capture_transcoder: TCaptureTranscoder,
    pub encoder: &'a mut TEncoder,
    pub stats: SendStats,
}

impl<'a, TCaptureSample, TCaptureDataReader, TCaptureTranscoder, TEncoder>
    SendService<'a, TCaptureSample, TCaptureDataReader, TCaptureTranscoder, TEncoder>
where
    TCaptureSample: Sample,
    TCaptureDataReader: AsyncReadItems<TCaptureSample>,
    TCaptureTranscoder: Transcode,
    TEncoder: Encoder<TCaptureSample, TCaptureDataReader> + ?Sized,
{
    pub async fn send_loop(
        &mut self,
        mut socket: UdpSocketSendHalf,
        peer_addr: SocketAddr,
    ) -> Result<futures::Never, crate::Error> {
        let mut send_buf = [0u8; SIZE];
        loop {
            trace!("Send loop begin");

            trace!("Send: before transcode");
            self.capture_transcoder.transcode().await?;
            trace!("Send: after transcode");

            trace!("Send: before encode");
            match self
                .encoder
                .encode(&mut self.capture_data_reader, &mut send_buf)
                .await
            {
                Ok(bytes_to_send) => {
                    trace!("Send: after encode, bytes to send: {}", bytes_to_send);
                    self.stats.frames_encoded += 1;
                    self.stats.bytes_encoded += bytes_to_send;

                    trace!("Send: before send_to");
                    let bytes_sent = socket
                        .send_to(&send_buf[..bytes_to_send], &peer_addr)
                        .await?;
                    trace!("Send: after send_to");
                    self.stats.packets_sent += 1;
                    self.stats.bytes_sent += bytes_sent;

                    if bytes_sent != bytes_to_send {
                        warn!(
                            "Send: sent {} bytes while expecting to send {} bytes ({} buffer size)",
                            bytes_sent,
                            bytes_to_send,
                            send_buf.len(),
                        );
                        self.stats.bytes_sent_mismatches += 1;
                    }

                    trace!("Send: sent a non-empty packet");
                }
                Err(EncodingError::NotEnoughData) => {
                    self.stats.not_enough_data_at_encoding_errors += 1;
                }
                Err(err) => {
                    error!("Encoding failed: {}", &err);
                    return Err(err)?;
                }
            };
            debug!("network send: {:?}", self.stats);
        }
    }
}
