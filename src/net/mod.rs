use mio::net::UdpSocket;
use mio::{Events, Poll, PollOpt, Ready, Token};
use std::time::Duration;

use crate::codec::{Decoder, DecodingError, Encoder};
use crate::samples::SharedSamples;

pub struct NetService<'a> {
    pub capture_buf: SharedSamples,
    pub playback_buf: SharedSamples,
    pub encoder: &'a mut dyn Encoder,
    pub decoder: &'a mut dyn Decoder,
}

impl<'a> NetService<'a> {
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

            if !ready_to_write_consumed {
                let mut capture_buf = self.capture_buf.lock();
                if !capture_buf.is_empty() {
                    match self.encoder.encode(&mut capture_buf, &mut send_buf) {
                        Ok(bytes_to_send) => {
                            ready_to_write_consumed = true;
                            let bytes_sent = socket.send(&send_buf[..bytes_to_send])?;

                            if bytes_sent != bytes_to_send {
                                println!(
                                    "sent {} bytes while expecting to send {} bytes ({} buffer size)",
                                    bytes_sent, bytes_to_send, send_buf.len(),
                                );
                            }

                            // println!(
                            //     "Sent a non-empty packet, capture buffer len: {}",
                            //     capture_buf.len()
                            // );
                        }
                        Err(err) => {
                            println!("Encoding failed: {}", &err);
                            return Err(err.into());
                        }
                    };
                } else {
                    println!(
                        "Attempted to send an empty packet, capture buffer len: {}",
                        capture_buf.len()
                    );
                }
            }

            if !ready_to_read_consumed {
                ready_to_read_consumed = true;
                let num_recv = socket.recv(&mut recv_buf)?;
                // println!("Read a packet of {} bytes", num_recv);

                if num_recv > 0 {
                    let mut playback_buf = self.playback_buf.lock();

                    match self
                        .decoder
                        .decode(&recv_buf[..num_recv], &mut playback_buf)
                    {
                        Ok(_num_samples) => {
                            // println!(
                            //     "Successfully decoded the packet into {} samples",
                            //     num_samples
                            // )
                        }
                        Err(DecodingError::EmptyPacket) => {
                            // noop
                        }
                        Err(err) => {
                            println!("Decoding failed: {}", &err);
                            return Err(err.into());
                        }
                    };
                } else {
                    // println!("Skipped processing of an empty incoming packet");
                }
            }
        }
    }
}
