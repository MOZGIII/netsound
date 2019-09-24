use async_std::net::UdpSocket;
use sample::Sample;
use std::net::SocketAddr;

use crate::codec::{Decoder, Encoder};
use crate::io::{ReadItems, WriteItems};
use crate::transcoder::Transcode;

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
    TCaptureData: ReadItems<TCaptureSample> + Transcode<TCaptureSample, TCaptureSample>,
    TPlaybackData: WriteItems<TPlaybackSample> + Transcode<TPlaybackSample, TPlaybackSample>,
    <TCaptureData as Transcode<TCaptureSample, TCaptureSample>>::Error:
        std::error::Error + std::marker::Send + std::marker::Sync,
    <TPlaybackData as Transcode<TPlaybackSample, TPlaybackSample>>::Error:
        std::error::Error + std::marker::Send + std::marker::Sync,
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

    TCaptureData: ReadItems<TCaptureSample> + Transcode<TCaptureSample, TCaptureSample> + Send,
    TPlaybackData: WriteItems<TPlaybackSample> + Transcode<TPlaybackSample, TPlaybackSample> + Send,

    <TCaptureData as Transcode<TCaptureSample, TCaptureSample>>::Error:
        std::error::Error + std::marker::Send + std::marker::Sync + 'static,
    <TPlaybackData as Transcode<TPlaybackSample, TPlaybackSample>>::Error:
        std::error::Error + std::marker::Send + std::marker::Sync + 'static,

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
