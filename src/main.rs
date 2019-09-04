#![warn(rust_2018_idioms, missing_debug_implementations)]

use std::env;
use std::net::SocketAddr;

use failure::Error;

use mio::net::UdpSocket;

mod audio;
mod audio_backends;
mod buf;
mod codec;
mod formats;
mod io;
mod net;
mod sync;

use audio::Backend;
use sync::synced;

fn main() -> Result<(), Error> {
    let bind_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());
    let send_addr = env::args().nth(2).unwrap_or_else(|| bind_addr.clone());
    let bind_addr: SocketAddr = bind_addr.parse()?;
    let send_addr: SocketAddr = send_addr.parse()?;

    let socket = UdpSocket::bind(&bind_addr)?;
    println!("Listening on: {}", socket.local_addr()?);
    println!("Sending to: {}", &send_addr);

    let capture_buf = synced(buf::VecDequeBuffer::with_capacity(30_000_000));
    let playback_buf = synced(buf::VecDequeBuffer::with_capacity(30_000_000));

    let audio_backend_builder = audio::BackendBuilder {
        capture_data_writer: capture_buf.clone(),
        playback_data_reader: playback_buf.clone(),

        request_capture_formats: formats::input(),
        request_playback_formats: formats::output(),
    };

    let backend_to_use = audio_backends::AudioBackendToUse::from_env()?;
    println!("Using audio backend: {:?}", backend_to_use);

    let codec_to_use = CodecToUse::from_env()?;
    println!("Using codec: {:?}", codec_to_use);

    let audio_backend = audio_backends::build(backend_to_use, audio_backend_builder)?;

    let capture_format = audio_backend.capture_format();
    let playback_format = audio_backend.playback_format();

    let mut encoder: Box<dyn codec::Encoder<f32, _>>;
    let mut decoder: Box<dyn codec::Decoder<f32, _>>;

    match codec_to_use {
        CodecToUse::Opus => {
            let opus_encoder_buf: Box<[f32]> = buffer(codec::opus::buf_size(
                capture_format,
                codec::opus::SupportedFrameSizeMS::F20,
                false,
            ));
            let opus_decoder_buf: Box<[f32]> = buffer(codec::opus::buf_size(
                playback_format,
                codec::opus::SupportedFrameSizeMS::F20,
                false,
            ));

            encoder = Box::new(codec::opus::make_encoder(capture_format, opus_encoder_buf)?);
            decoder = Box::new(codec::opus::make_decoder(
                playback_format,
                opus_decoder_buf,
            )?);
        }
        CodecToUse::Raw => {
            encoder = Box::new(codec::raw::Encoder);
            decoder = Box::new(codec::raw::Decoder);
        }
    };

    run_audio_backend(audio_backend)?;

    let mut net_service = net::NetService {
        capture_buf: capture_buf.clone(),
        playback_buf: playback_buf.clone(),
        encoder: &mut *encoder,
        decoder: &mut *decoder,
        stats: net::Stats::default(),
    };
    net_service.r#loop(socket, send_addr)?;

    Ok(())
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
