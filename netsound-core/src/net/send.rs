use crate::codec::{self, Encoder};
use crate::io::AsyncReadItems;
use crate::log::{debug, error, trace, warn, KV};
use crate::pcm::Sample;
use anyhow::format_err;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::{marker::PhantomData, sync::Arc};
use tokio::net::UdpSocket;

use super::SIZE;

mod multisend;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Default, Clone, Serialize, Deserialize, KV)]
pub struct SendStats {
    pub frames_encoded: usize,
    pub bytes_encoded: usize,
    pub not_enough_data_at_encoding_errors: usize,
    pub packets_sent: usize,
    pub bytes_sent: usize,
    pub bytes_sent_mismatches: usize,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct SendService<'a, TCaptureSample, TCaptureDataReader, TEncoder>
where
    TCaptureSample: Sample,
    TCaptureDataReader: AsyncReadItems<TCaptureSample>,
    TEncoder: Encoder<TCaptureSample, TCaptureDataReader> + ?Sized,
{
    pub capture_sample: PhantomData<TCaptureSample>,
    pub capture_data_reader: TCaptureDataReader,
    pub encoder: &'a mut TEncoder,
    pub stats: SendStats,
}

impl<'a, TCaptureSample, TCaptureDataReader, TEncoder>
    SendService<'a, TCaptureSample, TCaptureDataReader, TEncoder>
where
    TCaptureSample: Sample,
    TCaptureDataReader: AsyncReadItems<TCaptureSample>,
    TEncoder: Encoder<TCaptureSample, TCaptureDataReader> + ?Sized,
{
    pub async fn send_loop(
        &mut self,
        socket: Arc<UdpSocket>,
        peer_addrs: Vec<SocketAddr>,
    ) -> Result<futures::never::Never, crate::Error> {
        let mut send_buf = [0_u8; SIZE];
        loop {
            trace!("Send loop begin");

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
                    let bytes_sent = multisend::ensure_same_sizes(
                        multisend::multisend(
                            socket.as_ref(),
                            &send_buf[..bytes_to_send],
                            peer_addrs.iter(),
                        )
                        .await?,
                    )
                    .ok_or_else(|| format_err!("multisend has various results"))?;
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
                Err(codec::error::Encoding::NotEnoughData(_)) => {
                    self.stats.not_enough_data_at_encoding_errors += 1;
                }
                Err(err) => {
                    error!("Encoding failed: {}", &err);
                    return Err(err.into());
                }
            };
            debug!("network send"; &self.stats);
        }
    }
}
