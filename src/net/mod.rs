use mio::net::UdpSocket;
use mio::{Events, Poll, PollOpt, Ready, Token};
use sample::Sample;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::time::Duration;

use crate::codec::{Decoder, DecodingError, Encoder, EncodingError};
use crate::io::{ReadItems, WriteItems};
use crate::sync::Synced;
use crate::transcoder::Transcode;

#[derive(Debug, Default)]
pub struct Stats {
    pub send_ready_but_empty_capture_buf: usize,
    pub send_ready_but_try_lock_failed: usize,
    pub frames_encoded: usize,
    pub bytes_encoded: usize,
    pub not_enough_data_at_encoding_errors: usize,
    pub packets_sent: usize,
    pub bytes_sent: usize,
    pub bytes_sent_mismatches: usize,

    pub data_arrived_but_was_dropped_due_to_lock_conention: usize,
    pub packets_read: usize,
    pub bytes_read: usize,
    pub samples_decoded: usize,
    pub frames_decoded: usize,
    pub empty_packets_read: usize,
    pub empty_packets_decoding_errors: usize,
}

pub struct NetService<'a, TCaptureSample, TPlaybackSample, TCaptureData, TPlaybackData, TEnc, TDec>
where
    TCaptureSample: Sample,
    TPlaybackSample: Sample,
    TCaptureData: ReadItems<TCaptureSample> + Transcode<TCaptureSample, TCaptureSample>,
    TPlaybackData: WriteItems<TPlaybackSample> + Transcode<TPlaybackSample, TPlaybackSample>,
    <TCaptureData as Transcode<TCaptureSample, TCaptureSample>>::Error:
        std::error::Error + std::marker::Send + std::marker::Sync,
    <TPlaybackData as Transcode<TPlaybackSample, TPlaybackSample>>::Error:
        std::error::Error + std::marker::Send + std::marker::Sync,
    TEnc: Encoder<TCaptureSample, TCaptureData> + ?Sized,
    TDec: Decoder<TPlaybackSample, TPlaybackData> + ?Sized,
{
    pub capture_sample: PhantomData<TCaptureSample>,
    pub playback_sample: PhantomData<TPlaybackSample>,

    pub capture_data: Synced<TCaptureData>,
    pub playback_data: Synced<TPlaybackData>,
    pub encoder: &'a mut TEnc,
    pub decoder: &'a mut TDec,

    pub stats: Stats,
}

#[allow(dead_code)]
pub type DynNetService<'a, TCaptureSample, TPlaybackSample, TCaptureData, TPlaybackData> =
    NetService<
        'a,
        TCaptureSample,
        TPlaybackSample,
        TCaptureData,
        TPlaybackData,
        dyn Encoder<TCaptureSample, TCaptureData> + 'a,
        dyn Decoder<TPlaybackSample, TPlaybackData> + 'a,
    >;

impl<'a, TCaptureSample, TPlaybackSample, TCaptureData, TPlaybackData, TEnc, TDec>
    NetService<'a, TCaptureSample, TPlaybackSample, TCaptureData, TPlaybackData, TEnc, TDec>
where
    TCaptureSample: Sample,
    TPlaybackSample: Sample,

    TCaptureData: ReadItems<TCaptureSample> + Transcode<TCaptureSample, TCaptureSample>,
    TPlaybackData: WriteItems<TPlaybackSample> + Transcode<TPlaybackSample, TPlaybackSample>,

    <TCaptureData as Transcode<TCaptureSample, TCaptureSample>>::Error:
        std::error::Error + std::marker::Send + std::marker::Sync + 'static,
    <TPlaybackData as Transcode<TPlaybackSample, TPlaybackSample>>::Error:
        std::error::Error + std::marker::Send + std::marker::Sync + 'static,

    TEnc: Encoder<TCaptureSample, TCaptureData> + ?Sized,
    TDec: Decoder<TPlaybackSample, TPlaybackData> + ?Sized,
{
    pub fn r#loop(&mut self, socket: UdpSocket, peer_addr: SocketAddr) -> Result<(), crate::Error> {
        const SOCKET: Token = Token(0);

        let poll = Poll::new()?;

        poll.register(&socket, SOCKET, Ready::all(), PollOpt::edge())?;

        const SIZE: usize = 1024 * 4 * 2;
        let mut send_buf = [0u8; SIZE];
        let mut recv_buf = [0u8; SIZE];

        let mut events = Events::with_capacity(128);

        let mut ready_to_read_consumed = true;
        let mut ready_to_write_consumed = true;

        loop {
            poll.poll(&mut events, Some(Duration::from_millis(100)))?;

            for event in events.iter() {
                match event.token() {
                    SOCKET => {
                        if event.readiness().contains(Ready::writable()) {
                            ready_to_write_consumed = false;
                        }
                        if event.readiness().contains(Ready::readable()) {
                            ready_to_read_consumed = false;
                        }
                    }
                    _ => unreachable!(),
                }
            }

            let mut print_stats = false;

            if !ready_to_write_consumed {
                if let Some(mut capture_data) = self.capture_data.try_lock() {
                    capture_data.transcode()?;
                    if capture_data.items_available()? > 0 {
                        match self.encoder.encode(&mut *capture_data, &mut send_buf) {
                            Ok(bytes_to_send) => {
                                self.stats.frames_encoded += 1;
                                self.stats.bytes_encoded += bytes_to_send;

                                ready_to_write_consumed = true;
                                print_stats = true;
                                let bytes_sent =
                                    socket.send_to(&send_buf[..bytes_to_send], &peer_addr)?;
                                self.stats.packets_sent += 1;
                                self.stats.bytes_sent += bytes_sent;

                                if bytes_sent != bytes_to_send {
                                    println!(
                                        "sent {} bytes while expecting to send {} bytes ({} buffer size)",
                                        bytes_sent, bytes_to_send, send_buf.len(),
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
                        // println!(
                        //     "Attempted to send an empty packet, capture buffer len: {}",
                        //     capture_data.len()
                        // );
                        self.stats.send_ready_but_empty_capture_buf += 1;
                    }
                } else {
                    self.stats.send_ready_but_try_lock_failed += 1;
                }
            }

            if !ready_to_read_consumed {
                ready_to_read_consumed = true;
                print_stats = true;
                let num_recv = socket.recv(&mut recv_buf)?;
                // println!("Read a packet of {} bytes", num_recv);
                self.stats.packets_read += 1;
                self.stats.bytes_read += num_recv;

                if num_recv > 0 {
                    if let Some(mut playback_data) = self.playback_data.try_lock() {
                        match self
                            .decoder
                            .decode(&recv_buf[..num_recv], &mut *playback_data)
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
                        self.stats
                            .data_arrived_but_was_dropped_due_to_lock_conention += 1;
                    }
                } else {
                    // println!("Skipped processing of an empty incoming packet");
                    self.stats.empty_packets_read += 1;
                }
            }

            if print_stats {
                println!("network: {:?}", self.stats);
            }
        }
    }
}
