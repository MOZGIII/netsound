use crate::codec::{Decoder, Encoder};
use crate::io::{AsyncReadItems, AsyncWriteItems};
use crate::sample::Sample;
use crate::transcoder::Transcode;
use async_std::net::UdpSocket;
use std::net::SocketAddr;

mod recv;
mod send;

pub use recv::*;
pub use send::*;

pub struct NetService<
    'a,
    TCaptureSample,
    TPlaybackSample,
    TCaptureData,
    TPlaybackData,
    TEncoder,
    TDecoder,
> where
    TCaptureSample: Sample,
    TPlaybackSample: Sample,
    TCaptureData:
        AsyncReadItems<TCaptureSample> + Transcode<TCaptureSample, TCaptureSample> + Unpin,
    TPlaybackData:
        AsyncWriteItems<TPlaybackSample> + Transcode<TPlaybackSample, TPlaybackSample> + Unpin,
    <TCaptureData as Transcode<TCaptureSample, TCaptureSample>>::Error:
        std::error::Error + Send + Sync,
    <TPlaybackData as Transcode<TPlaybackSample, TPlaybackSample>>::Error:
        std::error::Error + Send + Sync,
    TEncoder: Encoder<TCaptureSample, TCaptureData> + ?Sized,
    TDecoder: Decoder<TPlaybackSample, TPlaybackData> + ?Sized,
{
    pub send_service: SendService<'a, TCaptureSample, TCaptureData, TEncoder>,
    pub recv_service: RecvService<'a, TPlaybackSample, TPlaybackData, TDecoder>,
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

type SharedSocket<T> = std::sync::Arc<T>;

fn shared_socket<T>(t: T) -> SharedSocket<T> {
    std::sync::Arc::new(t)
}

impl<'a, TCaptureSample, TPlaybackSample, TCaptureData, TPlaybackData, TEncoder, TDecoder>
    NetService<'a, TCaptureSample, TPlaybackSample, TCaptureData, TPlaybackData, TEncoder, TDecoder>
where
    TCaptureSample: Sample + Send,
    TPlaybackSample: Sample + Send,

    TCaptureData:
        AsyncReadItems<TCaptureSample> + Transcode<TCaptureSample, TCaptureSample> + Unpin + Send,
    TPlaybackData: AsyncWriteItems<TPlaybackSample>
        + Transcode<TPlaybackSample, TPlaybackSample>
        + Unpin
        + Send,

    <TCaptureData as Transcode<TCaptureSample, TCaptureSample>>::Error:
        std::error::Error + Send + Sync + 'static,
    <TPlaybackData as Transcode<TPlaybackSample, TPlaybackSample>>::Error:
        std::error::Error + Send + Sync + 'static,

    TEncoder: Encoder<TCaptureSample, TCaptureData> + Send + ?Sized,
    TDecoder: Decoder<TPlaybackSample, TPlaybackData> + Send + ?Sized,
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

        let (val, _) = futures::future::select(send_future, recv_future)
            .await
            .into_inner();

        val
    }
}
