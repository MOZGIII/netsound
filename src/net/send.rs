use async_std::net::UdpSocket;
use sample::Sample;
use std::marker::PhantomData;
use std::net::SocketAddr;

use crate::codec::{Encoder, EncodingError};
use crate::io::ReadItems;
use crate::sync::Synced;
use crate::transcoder::Transcode;

use super::*;

#[derive(Debug, Default)]
pub struct SendStats {
    pub send_ready_but_empty_capture_buf: usize,
    pub send_ready_but_try_lock_failed: usize,
    pub frames_encoded: usize,
    pub bytes_encoded: usize,
    pub not_enough_data_at_encoding_errors: usize,
    pub packets_sent: usize,
    pub bytes_sent: usize,
    pub bytes_sent_mismatches: usize,
}

pub struct SendService<'a, TCaptureSample, TCaptureData, TEncoder>
where
    TCaptureSample: Sample,
    TCaptureData: ReadItems<TCaptureSample> + Transcode<TCaptureSample, TCaptureSample>,
    <TCaptureData as Transcode<TCaptureSample, TCaptureSample>>::Error:
        std::error::Error + std::marker::Send + std::marker::Sync,
    TEncoder: Encoder<TCaptureSample, TCaptureData> + ?Sized,
{
    pub capture_sample: PhantomData<TCaptureSample>,
    pub capture_data: Synced<TCaptureData>,
    pub encoder: &'a mut TEncoder,
    pub stats: SendStats,
}

impl<'a, TCaptureSample, TCaptureData, TEncoder>
    SendService<'a, TCaptureSample, TCaptureData, TEncoder>
where
    TCaptureSample: Sample,
    TCaptureData: ReadItems<TCaptureSample> + Transcode<TCaptureSample, TCaptureSample>,
    <TCaptureData as Transcode<TCaptureSample, TCaptureSample>>::Error:
        std::error::Error + std::marker::Send + std::marker::Sync + 'static,
    TEncoder: Encoder<TCaptureSample, TCaptureData> + ?Sized,
{
    // TODO: for some reason, rustc detects this code as unreachable.
    #[allow(unreachable_code)]
    pub async fn send_loop(
        &mut self,
        socket: SharedSocket<UdpSocket>,
        peer_addr: SocketAddr,
    ) -> Result<futures::Never, crate::Error> {
        let mut send_buf = [0u8; SIZE];
        loop {
            let mut capture_data = self.capture_data.lock().await;

            capture_data.transcode()?;
            if capture_data.items_available()? > 0 {
                match self.encoder.encode(&mut *capture_data, &mut send_buf) {
                    Ok(bytes_to_send) => {
                        self.stats.frames_encoded += 1;
                        self.stats.bytes_encoded += bytes_to_send;

                        let bytes_sent = socket
                            .send_to(&send_buf[..bytes_to_send], &peer_addr)
                            .await?;
                        self.stats.packets_sent += 1;
                        self.stats.bytes_sent += bytes_sent;

                        if bytes_sent != bytes_to_send {
                            println!(
                                "sent {} bytes while expecting to send {} bytes ({} buffer size)",
                                bytes_sent,
                                bytes_to_send,
                                send_buf.len(),
                            );
                            self.stats.bytes_sent_mismatches += 1;
                        }

                        // println!(
                        //     "Sent a non-empty packet, capture buffer len: {}",
                        //     capture_data.len()
                        // );
                    }
                    Err(EncodingError::NotEnoughData) => {
                        self.stats.not_enough_data_at_encoding_errors += 1;
                    }
                    Err(err) => {
                        println!("Encoding failed: {}", &err);
                        return Err(err)?;
                    }
                };
            } else {
                self.stats.send_ready_but_empty_capture_buf += 1;
            }
            println!("network send: {:?}", self.stats);
        }
    }
}
