#![warn(rust_2018_idioms, missing_debug_implementations)]
#![feature(const_fn)]

use std::env;
use std::net::SocketAddr;

use failure::Error;

use futures::executor::block_on;
use tokio::{net::UdpSocket, runtime::Runtime};

use std::marker::PhantomData;

mod audio;
mod audio_backends;
mod buf;
mod codec;
mod format;
mod formats;
mod io;
mod match_channels;
mod net;
mod sample;
mod samples_filter;
mod transcoder;

use audio::Backend;

fn errmain() -> Result<(), Error> {
    let bind_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());
    let send_addr = env::args().nth(2).unwrap_or_else(|| bind_addr.clone());
    let bind_addr: SocketAddr = bind_addr.parse()?;
    let send_addr: SocketAddr = send_addr.parse()?;

    let socket = block_on(UdpSocket::bind(&bind_addr))?;
    println!("Listening on: {}", socket.local_addr()?);
    println!("Sending to: {}", &send_addr);

    let codec_to_use = CodecToUse::from_env()?;
    println!("Using codec: {:?}", codec_to_use);

    let backend_to_use = audio_backends::AudioBackendToUse::from_env()?;
    println!("Using audio backend: {:?}", backend_to_use);

    let audio_backend_build_params = audio_backends::BuildParams {
        request_capture_formats: formats::input(),
        request_playback_formats: formats::output(),
    };
    let (negotiated_formats, continuation) =
        audio_backends::negotiate_formats(backend_to_use, audio_backend_build_params)?;

    use std::convert::TryInto;
    let (capture_transcoder, capture_data_writer, capture_data_reader) = {
        let format = negotiated_formats.capture_format;
        let channels = format.channels.try_into()?;
        let (audio_writer, transcoder_reader) = buf::vec_deque_buffer_with_capacity(30_000_000);
        let (trascoder_writer, net_reader) = buf::vec_deque_buffer_with_capacity(30_000_000);
        (
            transcoder::resampler::Resampler::new(
                channels,
                std::cmp::min(2, channels),
                format.sample_rate.into(),
                48000.0,
                transcoder_reader,
                trascoder_writer,
            ),
            audio_writer,
            net_reader,
        )
    };
    let (playback_transcoder, playback_data_writer, playback_data_reader) = {
        let format = negotiated_formats.capture_format;
        let channels = format.channels.try_into()?;
        let (net_writer, transcoder_reader) = buf::vec_deque_buffer_with_capacity(30_000_000);
        let (trascoder_writer, audio_reader) = buf::vec_deque_buffer_with_capacity(30_000_000);
        (
            transcoder::resampler::Resampler::new(
                std::cmp::min(2, channels),
                channels,
                48000.0,
                format.sample_rate.into(),
                transcoder_reader,
                trascoder_writer,
            ),
            net_writer,
            audio_reader,
        )
    };

    let resampled_capture_format = format::Format::new(
        std::cmp::min(2, negotiated_formats.capture_format.channels),
        48000,
    );
    let resampled_playback_format = format::Format::new(
        std::cmp::min(2, negotiated_formats.playback_format.channels),
        48000,
    );

    let mut encoder: Box<dyn codec::Encoder<f32, _> + Send>;
    let mut decoder: Box<dyn codec::Decoder<f32, _> + Send>;

    match codec_to_use {
        CodecToUse::Opus => {
            let opus_encoder_buf: Box<[f32]> = buffer(codec::opus::buf_size(
                resampled_capture_format.sample_rate,
                resampled_capture_format.channels,
                codec::opus::SupportedFrameSizeMS::F20,
                false,
            ));
            let opus_decoder_buf: Box<[f32]> = buffer(codec::opus::buf_size(
                resampled_playback_format.sample_rate,
                resampled_playback_format.channels,
                codec::opus::SupportedFrameSizeMS::F20,
                false,
            ));

            encoder = Box::new(codec::opus::make_encoder(
                resampled_capture_format,
                opus_encoder_buf,
            )?);
            decoder = Box::new(codec::opus::make_decoder(
                resampled_playback_format,
                opus_decoder_buf,
            )?);
        }
        CodecToUse::Raw => {
            encoder = Box::new(codec::raw::Encoder);
            decoder = Box::new(codec::raw::Decoder);
        }
    };

    let audio_backend = continuation(capture_data_writer, playback_data_reader)?;
    run_audio_backend(audio_backend)?;

    let mut net_service = net::NetService {
        send_service: net::SendService {
            capture_sample: PhantomData,
            capture_data_reader,
            capture_transcoder,
            encoder: &mut *encoder,
            stats: net::SendStats::default(),
        },
        recv_service: net::RecvService {
            playback_sample: PhantomData,
            playback_data_writer,
            playback_transcoder,
            decoder: &mut *decoder,
            stats: net::RecvStats::default(),
        },
    };

    let rt = Runtime::new()?;
    rt.block_on(net_service.net_loop(socket, send_addr))?;

    Ok(())
}

fn main() {
    if let Err(err) = errmain() {
        eprintln!("Error: {} [{:?}]", err, err);
        std::process::exit(1);
    }
}

#[derive(Debug)]
enum CodecToUse {
    Opus,
    Raw,
}

impl CodecToUse {
    fn from_env() -> Result<Self, std::env::VarError> {
        Ok(match std::env::var("CODEC") {
            Ok(ref val) if val == "opus" => CodecToUse::Opus,
            Ok(ref val) if val == "raw" => CodecToUse::Raw,
            // Defaults.
            Ok(_) | Err(std::env::VarError::NotPresent) => CodecToUse::Opus,
            // Invalid value.
            Err(e) => return Err(e),
        })
    }
}

fn run_audio_backend(audio_backend: Box<dyn Backend + 'static>) -> Result<(), Error> {
    std::thread::spawn(move || {
        let mut local = audio_backend;
        local.run()
    });
    Ok(())
}

fn buffer<T: Default + Clone>(size: usize) -> Box<[T]> {
    let mut vec = Vec::with_capacity(size);
    let cap = vec.capacity();
    vec.resize(cap, T::default());
    vec.into()
}
