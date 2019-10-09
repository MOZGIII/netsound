use crate::codec::{Decoder, DecodingError};
use crate::io::AsyncWriteItems;
use crate::log::*;
use crate::sample::Sample;
use crate::transcode::Transcode;
use std::marker::PhantomData;
use tokio::net::udp::split::UdpSocketRecvHalf;

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

pub struct RecvService<'a, TPlaybackSample, TPlaybackDataWriter, TPlaybackTranscoder, TDecoder>
where
    TPlaybackSample: Sample,
    TPlaybackDataWriter: AsyncWriteItems<TPlaybackSample>,
    TPlaybackTranscoder: Transcode,
    TDecoder: Decoder<TPlaybackSample, TPlaybackDataWriter> + ?Sized,
{
    pub playback_sample: PhantomData<TPlaybackSample>,
    pub playback_data_writer: TPlaybackDataWriter,
    pub playback_transcoder: TPlaybackTranscoder,
    pub decoder: &'a mut TDecoder,
    pub stats: RecvStats,
}

impl<'a, TPlaybackSample, TPlaybackDataWriter, TPlaybackTranscoder, TDecoder>
    RecvService<'a, TPlaybackSample, TPlaybackDataWriter, TPlaybackTranscoder, TDecoder>
where
    TPlaybackSample: Sample,
    TPlaybackDataWriter: AsyncWriteItems<TPlaybackSample> + Unpin,
    TPlaybackTranscoder: Transcode,
    TDecoder: Decoder<TPlaybackSample, TPlaybackDataWriter> + ?Sized,
{
    pub async fn recv_loop(
        &mut self,
        mut socket: UdpSocketRecvHalf,
    ) -> Result<futures::Never, crate::Error> {
        let mut recv_buf = [0u8; SIZE];
        loop {
            trace!("Recv loop begin");

            trace!("Recv: before recv");
            let num_recv = socket.recv(&mut recv_buf).await?;
            trace!("Recv: after recv, read a packet of {} bytes", num_recv);
            self.stats.packets_read += 1;
            self.stats.bytes_read += num_recv;

            if num_recv > 0 {
                trace!("Recv: before decode");
                match self
                    .decoder
                    .decode(&recv_buf[..num_recv], &mut self.playback_data_writer)
                    .await
                {
                    Ok(num_samples) => {
                        trace!("Recv: after decode, samples decoded: {}", num_samples);
                        self.stats.samples_decoded += num_samples;
                        self.stats.frames_decoded += 1;

                        trace!("Recv: before transcode");
                        self.playback_transcoder.transcode().await?;
                        trace!("Recv: after transcode");
                    }
                    Err(DecodingError::EmptyPacket) => {
                        self.stats.empty_packets_decoding_errors += 1;
                        // noop
                    }
                    Err(err) => {
                        error!("Decoding failed: {}", &err);
                        return Err(err)?;
                    }
                };
            } else {
                warn!("Recv: skipped processing of an empty incoming packet");
                self.stats.empty_packets_read += 1;
            }
            debug!("network recv: {:?}", self.stats);
        }
    }
}
