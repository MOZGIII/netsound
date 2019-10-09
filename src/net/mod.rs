use crate::codec::{Decoder, Encoder};
use crate::io::{AsyncReadItems, AsyncWriteItems};
use crate::log::*;
use crate::sample::Sample;
use crate::UdpSocket;
use std::net::SocketAddr;

mod recv;
mod send;

pub use recv::*;
pub use send::*;

pub struct NetService<
    'a,
    TCaptureSample,
    TPlaybackSample,
    TCaptureDataReader,
    TPlaybackDataWriter,
    TEncoder,
    TDecoder,
> where
    TCaptureSample: Sample,
    TPlaybackSample: Sample,

    TCaptureDataReader: AsyncReadItems<TCaptureSample> + Unpin,
    TPlaybackDataWriter: AsyncWriteItems<TPlaybackSample> + Unpin,

    TEncoder: Encoder<TCaptureSample, TCaptureDataReader> + ?Sized,
    TDecoder: Decoder<TPlaybackSample, TPlaybackDataWriter> + ?Sized,
{
    pub send_service: SendService<'a, TCaptureSample, TCaptureDataReader, TEncoder>,
    pub recv_service: RecvService<'a, TPlaybackSample, TPlaybackDataWriter, TDecoder>,
}

#[allow(dead_code)]
pub type DynNetService<'a, TCaptureSample, TPlaybackSample, TCaptureData, TPlaybackData> =
    NetService<
        'a,
        TCaptureSample,
        TPlaybackSample,
        TCaptureData,
        TPlaybackData,
        dyn Encoder<TCaptureSample, TCaptureData> + Send + 'a,
        dyn Decoder<TPlaybackSample, TPlaybackData> + Send + 'a,
    >;

const SIZE: usize = 1024 * 4 * 2;

impl<
        'a,
        TCaptureSample,
        TPlaybackSample,
        TCaptureDataReader,
        TPlaybackDataWriter,
        TEncoder,
        TDecoder,
    >
    NetService<
        'a,
        TCaptureSample,
        TPlaybackSample,
        TCaptureDataReader,
        TPlaybackDataWriter,
        TEncoder,
        TDecoder,
    >
where
    TCaptureSample: Sample + Send,
    TPlaybackSample: Sample + Send,

    TCaptureDataReader: AsyncReadItems<TCaptureSample> + Unpin + Send,
    TPlaybackDataWriter: AsyncWriteItems<TPlaybackSample> + Unpin + Send,

    TEncoder: Encoder<TCaptureSample, TCaptureDataReader> + Send + ?Sized,
    TDecoder: Decoder<TPlaybackSample, TPlaybackDataWriter> + Send + ?Sized,
{
    pub async fn net_loop(
        &mut self,
        socket: UdpSocket,
        peer_addr: SocketAddr,
    ) -> Result<futures::Never, crate::Error> {
        let send_service = &mut self.send_service;
        let recv_service = &mut self.recv_service;

        let (socket_recv_half, socket_send_half) = socket.split();

        use futures::FutureExt;
        let send_future = send_service.send_loop(socket_send_half, peer_addr).boxed();
        let recv_future = recv_service.recv_loop(socket_recv_half).boxed();

        let (val, _) = futures::future::select(recv_future, send_future)
            .await
            .into_inner();

        debug!("net loop finished");

        val
    }
}
