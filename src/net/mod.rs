use crate::codec::{Decoder, Encoder};
use crate::io::{AsyncReadItems, AsyncWriteItems};
use crate::sample::Sample;
use crate::transcoder::Transcode;
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
    TCaptureTranscoder,
    TPlaybackTranscoder,
    TEncoder,
    TDecoder,
> where
    TCaptureSample: Sample,
    TPlaybackSample: Sample,

    TCaptureDataReader: AsyncReadItems<TCaptureSample> + Unpin,
    TPlaybackDataWriter: AsyncWriteItems<TPlaybackSample> + Unpin,

    TCaptureTranscoder: Transcode + Unpin,
    TPlaybackTranscoder: Transcode + Unpin,

    <TCaptureTranscoder as Transcode>::Error: std::error::Error + Send + Sync,
    <TPlaybackTranscoder as Transcode>::Error: std::error::Error + Send + Sync,

    TEncoder: Encoder<TCaptureSample, TCaptureDataReader> + ?Sized,
    TDecoder: Decoder<TPlaybackSample, TPlaybackDataWriter> + ?Sized,
{
    pub send_service:
        SendService<'a, TCaptureSample, TCaptureDataReader, TCaptureTranscoder, TEncoder>,
    pub recv_service:
        RecvService<'a, TPlaybackSample, TPlaybackDataWriter, TPlaybackTranscoder, TDecoder>,
}

#[allow(dead_code)]
pub type DynNetService<
    'a,
    TCaptureSample,
    TPlaybackSample,
    TCaptureData,
    TPlaybackData,
    TCaptureTranscoder,
    TPlaybackTranscoder,
> = NetService<
    'a,
    TCaptureSample,
    TPlaybackSample,
    TCaptureData,
    TPlaybackData,
    TCaptureTranscoder,
    TPlaybackTranscoder,
    dyn Encoder<TCaptureSample, TCaptureData> + Send + 'a,
    dyn Decoder<TPlaybackSample, TPlaybackData> + Send + 'a,
>;

const SIZE: usize = 1024 * 4 * 2;

type SharedSocket<T> = std::sync::Arc<T>;

fn shared_socket<T>(t: T) -> SharedSocket<T> {
    std::sync::Arc::new(t)
}

impl<
        'a,
        TCaptureSample,
        TPlaybackSample,
        TCaptureDataReader,
        TPlaybackDataWriter,
        TCaptureTranscoder,
        TPlaybackTranscoder,
        TEncoder,
        TDecoder,
    >
    NetService<
        'a,
        TCaptureSample,
        TPlaybackSample,
        TCaptureDataReader,
        TPlaybackDataWriter,
        TCaptureTranscoder,
        TPlaybackTranscoder,
        TEncoder,
        TDecoder,
    >
where
    TCaptureSample: Sample + Send,
    TPlaybackSample: Sample + Send,

    TCaptureDataReader: AsyncReadItems<TCaptureSample> + Unpin + Send,
    TPlaybackDataWriter: AsyncWriteItems<TPlaybackSample> + Unpin + Send,

    TCaptureTranscoder: Transcode + Unpin + Send,
    TPlaybackTranscoder: Transcode + Unpin + Send,

    <TCaptureTranscoder as Transcode>::Error: std::error::Error + Send + Sync + 'static,
    <TPlaybackTranscoder as Transcode>::Error: std::error::Error + Send + Sync + 'static,

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
        let socket = shared_socket(socket);

        use futures::FutureExt;
        let send_future = send_service.send_loop(socket.clone(), peer_addr).boxed();
        let recv_future = recv_service.recv_loop(socket).boxed();

        let (val, _) = futures::future::select(recv_future, send_future)
            .await
            .into_inner();

        println!("net loop finished");

        val
    }
}
