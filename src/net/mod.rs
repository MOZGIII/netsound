use mio::net::UdpSocket;
use mio::{Events, Poll, PollOpt, Ready, Token};
use std::time::Duration;

use crate::codec::{Decoder, DecodingError, Encoder, EncodingError};
use crate::samples::SharedSamples;

#[derive(Debug, Default)]
pub struct Stats {
    pub send_ready_but_empty_capture_buf: usize,
    pub frames_encoded: usize,
    pub bytes_encoded: usize,
    pub not_enough_data_at_encoding_errors: usize,
    pub packets_sent: usize,
    pub bytes_sent: usize,
    pub bytes_sent_mismatches: usize,

    pub packets_read: usize,
    pub bytes_read: usize,
    pub samples_decoded: usize,
    pub frames_decoded: usize,
    pub empty_packets_read: usize,
    pub empty_packets_decoding_errors: usize,
}

pub struct NetService<'a, E, D>
where
    E: Encoder + ?Sized,
    D: Decoder + ?Sized,
{
    pub capture_buf: SharedSamples,
    pub playback_buf: SharedSamples,
    pub encoder: &'a mut E,
    pub decoder: &'a mut D,

    pub stats: Stats,
}

#[allow(dead_code)]
pub type DynNetService<'a> = NetService<'a, dyn Encoder + 'a, dyn Decoder + 'a>;

impl<'a, E, D> NetService<'a, E, D>
where
    E: Encoder + ?Sized,
    D: Decoder + ?Sized,
{
    pub fn r#loop(&mut self, socket: UdpSocket) -> Result<(), Box<dyn std::error::Error>> {
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
                let mut capture_buf = self.capture_buf.lock();
                if !capture_buf.is_empty() {
                    match self.encoder.encode(&mut capture_buf, &mut send_buf) {
                        Ok(bytes_to_send) => {
                            self.stats.frames_encoded += 1;
                            self.stats.bytes_encoded += bytes_to_send;

                            ready_to_write_consumed = true;
                            print_stats = true;
                            let bytes_sent = socket.send(&send_buf[..bytes_to_send])?;
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
                            //     capture_buf.len()
                            // );
                        }
                        Err(EncodingError::NotEnoughData) => {
                            self.stats.not_enough_data_at_encoding_errors += 1;
                        }
                        Err(err) => {
                            println!("Encoding failed: {}", &err);
                            return Err(err.into());
                        }
                    };
                } else {
                    // println!(
                    //     "Attempted to send an empty packet, capture buffer len: {}",
                    //     capture_buf.len()
                    // );
                    self.stats.send_ready_but_empty_capture_buf += 1;
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
                    let mut playback_buf = self.playback_buf.lock();

                    match self
                        .decoder
                        .decode(&recv_buf[..num_recv], &mut playback_buf)
                    {
                        Ok(num_samples) => {
                            // println!(
                            //     "Successfully decoded the packet into {} samples",
                            //     num_samples
                            // )
                            self.stats.samples_decoded += num_samples;
                            self.stats.frames_decoded += 1;
                        }
                        Err(DecodingError::EmptyPacket) => {
                            self.stats.empty_packets_decoding_errors += 1;
                            // noop
                        }
                        Err(err) => {
                            println!("Decoding failed: {}", &err);
                            return Err(err.into());
                        }
                    };
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
