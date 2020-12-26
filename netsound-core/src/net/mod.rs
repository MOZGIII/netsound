use crate::codec::{Decoder, Encoder};
use crate::io::{AsyncReadItems, AsyncWriteItems};
use crate::log::{debug, logger, o, LogScopeFutureExt};
use crate::pcm::Sample;
use futures::{future::select, FutureExt};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::UdpSocket;

mod recv;
mod send;

pub use recv::*;
pub use send::*;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
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
        peer_addrs: Vec<SocketAddr>,
    ) -> Result<futures::never::Never, crate::Error> {
        let send_service = &mut self.send_service;
        let recv_service = &mut self.recv_service;

        let socket = Arc::new(socket);

        let send_future = send_service
            .send_loop(socket.clone(), peer_addrs)
            .with_logger(logger().new(o!("logger" => "net::send")))
            .boxed();
        let recv_future = recv_service
            .recv_loop(socket)
            .with_logger(logger().new(o!("logger" => "net::recv")))
            .boxed();

        let val = select(recv_future, send_future).await.factor_first().0;

        debug!("net loop finished");

        val
    }
}
