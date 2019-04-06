use mio::net::UdpSocket;
use mio::{Events, Poll, PollOpt, Ready, Token};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;

use crate::samples::Samples;

mod udp_codec;

pub struct NetService {
    pub capture_buf: Arc<Mutex<Samples>>,
    pub playback_buf: Arc<Mutex<Samples>>,
}

impl NetService {
    pub fn r#loop(&self, socket: UdpSocket) -> Result<(), std::io::Error> {
        const SOCKET: Token = Token(0);

        let poll = Poll::new()?;

        poll.register(&socket, SOCKET, Ready::all(), PollOpt::edge())?;

        let mut send_buf = [0u8; 1024];
        let mut recv_buf = [0u8; 1024];

        let mut events = Events::with_capacity(128);
        loop {
            poll.poll(&mut events, Some(Duration::from_millis(100)))?;
            for event in events.iter() {
                match event.token() {
                    SOCKET => {
                        if event.readiness().contains(Ready::writable()) {
                            let mut capture_buf = self.capture_buf.lock();
                            let bytes_to_send =
                                udp_codec::vecdec_to_sendbuf(&mut capture_buf, &mut send_buf);

                            let bytes_sent = socket.send(&send_buf[..bytes_to_send])?;

                            if bytes_sent != bytes_to_send {
                                println!(
                                    "sent {} bytes while expecting to send {} bytes ({} buffer size)",
                                    bytes_sent, bytes_to_send, send_buf.len(),
                                );
                            }
                        }
                        if event.readiness().contains(Ready::readable()) {
                            let num_recv = socket.recv(&mut recv_buf)?;
                            let mut playback_buf = self.playback_buf.lock();
                            udp_codec::recvbuf_to_vecdec(&recv_buf[..num_recv], &mut playback_buf);
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}
