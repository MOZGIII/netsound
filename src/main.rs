#![warn(rust_2018_idioms, missing_debug_implementations)]
#![feature(const_fn)]

use std::env;
use std::net::SocketAddr;

use failure::Error;

use mio::net::UdpSocket;

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
mod samples_filter;
mod sync;
mod transcoder;

use audio::Backend;

fn errmain() -> Result<(), Error> {
    let bind_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());
    let send_addr = env::args().nth(2).unwrap_or_else(|| bind_addr.clone());
    let bind_addr: SocketAddr = bind_addr.parse()?;
    let send_addr: SocketAddr = send_addr.parse()?;

    let socket = UdpSocket::bind(&bind_addr)?;
    println!("Listening on: {}", socket.local_addr()?);
    println!("Sending to: {}", &send_addr);

    let codec_to_use = CodecToUse::from_env()?;
    println!("Using codec: {:?}", codec_to_use);

    let backend_to_use = audio_backends::AudioBackendToUse::from_env()?;
    println!("Using audio backend: {:?}", backend_to_use);

    use std::convert::TryInto;
    let audio_backend_builder = audio::Builder {
        request_capture_formats: formats::input(),
        request_playback_formats: formats::output(),
        shared_capture_data_builder: |f| {
            let channels = f.channels.try_into()?;
            Ok(sync::synced(transcoder::resampler::Resampler::new(
                channels,
                std::cmp::min(2, channels),
                f.sample_rate.into(),
                48000.0,
                buf::VecDequeBuffer::with_capacity(30_000_000),
                buf::VecDequeBuffer::with_capacity(30_000_000),
            )))
        },
        shared_playback_data_builder: |f| {
            let channels = f.channels.try_into()?;
            Ok(sync::synced(transcoder::resampler::Resampler::new(
                std::cmp::min(2, channels),
                channels,
                48000.0,
                f.sample_rate.into(),
                buf::VecDequeBuffer::with_capacity(30_000_000),
                buf::VecDequeBuffer::with_capacity(30_000_000),
            )))
        },
    };

    let audio_state = audio_backends::build(backend_to_use, audio_backend_builder)?;

    let resampled_capture_format =
        format::Format::new(std::cmp::min(2, audio_state.capture_format.channels), 48000);
    let resampled_playback_format = format::Format::new(
        std::cmp::min(2, audio_state.playback_format.channels),
        48000,
    );

    let mut encoder: Box<dyn codec::Encoder<f32, _>>;
    let mut decoder: Box<dyn codec::Decoder<f32, _>>;

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

    let audio_backend = audio_state.backend;
    run_audio_backend(audio_backend)?;

    let mut net_service = net::NetService {
        capture_sample: PhantomData,
        playback_sample: PhantomData,
        capture_data: audio_state.shared_capture_data.clone(),
        playback_data: audio_state.shared_playback_data.clone(),
        encoder: &mut *encoder,
        decoder: &mut *decoder,
        stats: net::Stats::default(),
    };
    net_service.r#loop(socket, send_addr)?;

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
