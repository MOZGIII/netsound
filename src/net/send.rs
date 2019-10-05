use crate::codec::{Encoder, EncodingError};
use crate::io::AsyncReadItems;
use crate::sample::Sample;
use crate::transcoder::Transcode;
use crate::UdpSocket;
use std::marker::PhantomData;
use std::net::SocketAddr;

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
        socket: SharedSocket<UdpSocket>,
        peer_addr: SocketAddr,
    ) -> Result<futures::Never, crate::Error> {
        let mut send_buf = [0u8; SIZE];
        loop {
            self.capture_transcoder.transcode().await?;
            match self
                .encoder
                .encode(&mut self.capture_data_reader, &mut send_buf)
                .await
            {
                Ok(bytes_to_send) => {
                    self.stats.frames_encoded += 1;
                    self.stats.bytes_encoded += bytes_to_send;

                    println!("Before send_to");
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
            println!("network send: {:?}", self.stats);
        }
    }
}
