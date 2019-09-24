use crate::codec::{Decoder, DecodingError};
use crate::io::AsyncWriteItems;
use crate::sample::Sample;
use crate::sync::Synced;
use crate::transcoder::Transcode;
use async_std::net::UdpSocket;
use std::marker::PhantomData;

use super::*;

#[derive(Debug, Default)]
pub struct RecvStats {
    pub data_arrived_but_was_dropped_due_to_lock_conention: usize,
    pub packets_read: usize,
    pub bytes_read: usize,
    pub samples_decoded: usize,
    pub frames_decoded: usize,
    pub empty_packets_read: usize,
    pub empty_packets_decoding_errors: usize,
}

pub struct RecvService<'a, TPlaybackSample, TPlaybackData, TDecoder>
where
    TPlaybackSample: Sample,
    TPlaybackData:
        AsyncWriteItems<TPlaybackSample> + Transcode<TPlaybackSample, TPlaybackSample> + Unpin,
    <TPlaybackData as Transcode<TPlaybackSample, TPlaybackSample>>::Error:
        std::error::Error + Send + Sync,
    TDecoder: Decoder<TPlaybackSample, TPlaybackData> + ?Sized,
{
    pub playback_sample: PhantomData<TPlaybackSample>,
    pub playback_data: Synced<TPlaybackData>,
    pub decoder: &'a mut TDecoder,
    pub stats: RecvStats,
}

impl<'a, TPlaybackSample, TPlaybackData, TDecoder>
    RecvService<'a, TPlaybackSample, TPlaybackData, TDecoder>
where
    TPlaybackSample: Sample,
    TPlaybackData:
        AsyncWriteItems<TPlaybackSample> + Transcode<TPlaybackSample, TPlaybackSample> + Unpin,
    <TPlaybackData as Transcode<TPlaybackSample, TPlaybackSample>>::Error:
        std::error::Error + Send + Sync + 'static,
    TDecoder: Decoder<TPlaybackSample, TPlaybackData> + ?Sized,
{
    // TODO: for some reason, rustc detects this code as unreachable.
    #[allow(unreachable_code)]
    pub async fn recv_loop(
        &mut self,
        socket: SharedSocket<UdpSocket>,
    ) -> Result<futures::Never, crate::Error> {
        let mut recv_buf = [0u8; SIZE];
        loop {
            let num_recv = socket.recv(&mut recv_buf).await?;
            // println!("Read a packet of {} bytes", num_recv);
            self.stats.packets_read += 1;
            self.stats.bytes_read += num_recv;

            if num_recv > 0 {
                let mut playback_data = self.playback_data.lock().await;
                match self
                    .decoder
                    .decode(&recv_buf[..num_recv], &mut *playback_data)
                    .await
                {
                    Ok(num_samples) => {
                        // println!(
                        //     "Successfully decoded the packet into {} samples",
                        //     num_samples
                        // )
                        self.stats.samples_decoded += num_samples;
                        self.stats.frames_decoded += 1;
                        playback_data.transcode()?;
                    }
                    Err(DecodingError::EmptyPacket) => {
                        self.stats.empty_packets_decoding_errors += 1;
                        // noop
                    }
                    Err(err) => {
                        println!("Decoding failed: {}", &err);
                        return Err(err)?;
                    }
                };
            } else {
                // println!("Skipped processing of an empty incoming packet");
                self.stats.empty_packets_read += 1;
            }
            println!("network recv: {:?}", self.stats);
        }
    }
}
