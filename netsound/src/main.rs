#![warn(rust_2018_idioms, missing_debug_implementations)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use futures::FutureExt;
use std::convert::TryInto;
use std::env;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::{net::UdpSocket, runtime::Runtime};

use netsound_core::{
    audio_backend, buf, codec, future, io, log, net, pcm, transcode, transcode_service, Error,
};

mod audio_backends;
mod stream_configs;

use audio_backend::Backend;
use future::select_first;
use log::{info, logger, o, slog_info, warn, LogScopeFutureExt};

type DynTranscoder = Box<dyn transcode::Transcode<Ok = futures::never::Never> + Send>;

#[allow(clippy::too_many_lines)]
fn errmain() -> Result<(), Error> {
    let mut logger_cfg = slog_env_cfg::config_from_env()?;
    logger_cfg.env_logger_override_default_filter = Some("trace".to_string());
    let logger_root = slog::Logger::root(logger_cfg.build(), o![]);
    let _logger_guard = slog_scope::set_global_logger(logger_root);

    let mut args = env::args().skip(1);

    let bind_addr = args.next().unwrap_or_else(|| "127.0.0.1:8080".to_string());
    let send_addrs = {
        let vec: Vec<_> = args.collect();
        if vec.is_empty() {
            vec![bind_addr.clone()]
        } else {
            vec
        }
    };
    let bind_addr: SocketAddr = bind_addr.parse()?;
    let send_addrs = {
        let result: Result<Vec<SocketAddr>, <SocketAddr as FromStr>::Err> =
            send_addrs.into_iter().map(|e| e.parse()).collect();
        result?
    };

    let mut rt = Runtime::new()?;

    let socket = rt.block_on(UdpSocket::bind(&bind_addr))?;
    slog_info!(logger(), "Listening on: {}", socket.local_addr()?);
    info!("Sending to: {:?}", &send_addrs);

    let codec_to_use = CodecToUse::from_env()?;
    info!("Using codec: {:?}", codec_to_use);

    let backend_to_use = audio_backends::AudioBackendToUse::from_env()?;
    info!("Using audio backend: {:?}", backend_to_use);

    let audio_backend_build_params = audio_backends::BuildParams {
        request_capture_stream_configs: stream_configs::input(),
        request_playback_stream_configs: stream_configs::output(),
        logger: logger().new(o!("logger" => "audio")),
    };
    let (negotiated_stream_configs, continuation) =
        audio_backends::negotiate_stream_configs(&backend_to_use, audio_backend_build_params)?;

    let net_capture_stream_config = pcm::StreamConfig::new(
        48000.into(),
        std::cmp::min(2, negotiated_stream_configs.capture.channels()),
    );
    let net_playback_stream_config = pcm::StreamConfig::new(
        48000.into(),
        std::cmp::min(2, negotiated_stream_configs.playback.channels()),
    );

    let (capture_transcoder, capture_data_writer, capture_data_reader) = {
        let audio_stream_config = &negotiated_stream_configs.capture;
        let net_stream_config = &net_capture_stream_config;

        if audio_stream_config == net_stream_config {
            info!(
                "capture transcoder is noop: {} => {}",
                audio_stream_config, net_stream_config,
            );

            let (audio_writer, net_reader) = buf::vec_deque_buffer_with_capacity(30_000_000);
            (
                Box::new(transcode::noop::Noop) as DynTranscoder,
                audio_writer,
                net_reader,
            )
        } else {
            info!(
                "capture resampler config: {} => {}",
                audio_stream_config, net_stream_config,
            );

            let audio_channels = audio_stream_config.channels();
            let net_channels = net_stream_config.channels();
            let audio_sample_rate = {
                let val: u32 = audio_stream_config.sample_rate().as_usize().try_into()?;
                val.into()
            };
            let net_sample_rate = {
                let val: u32 = net_stream_config.sample_rate().as_usize().try_into()?;
                val.into()
            };

            let (audio_writer, transcoder_reader) = buf::vec_deque_buffer_with_capacity(30_000_000);
            let (transcoder_writer, net_reader) = buf::vec_deque_buffer_with_capacity(30_000_000);

            (
                Box::new(transcode::resampler::Resampler::new(
                    audio_channels,
                    net_channels,
                    audio_sample_rate,
                    net_sample_rate,
                    transcoder_reader,
                    transcoder_writer,
                )) as DynTranscoder,
                audio_writer,
                net_reader,
            )
        }
    };
    let (playback_transcoder, playback_data_writer, playback_data_reader) = {
        let net_stream_config = &net_playback_stream_config;
        let audio_stream_config = &negotiated_stream_configs.playback;

        if net_stream_config == audio_stream_config {
            info!(
                "playback transcoder is noop: {} => {}",
                net_stream_config, audio_stream_config,
            );

            let (net_writer, audio_reader) = buf::vec_deque_buffer_with_capacity(30_000_000);
            (
                Box::new(transcode::noop::Noop) as DynTranscoder,
                net_writer,
                audio_reader,
            )
        } else {
            info!(
                "playback resampler config: {} => {}",
                net_stream_config, audio_stream_config,
            );

            let net_channels = net_stream_config.channels();
            let audio_channels = audio_stream_config.channels();
            let net_sample_rate = {
                let val: u32 = net_stream_config.sample_rate().as_usize().try_into()?;
                val.into()
            };
            let audio_sample_rate = {
                let val: u32 = audio_stream_config.sample_rate().as_usize().try_into()?;
                val.into()
            };

            let (net_writer, transcoder_reader) = buf::vec_deque_buffer_with_capacity(30_000_000);
            let (transcoder_writer, audio_reader) = buf::vec_deque_buffer_with_capacity(30_000_000);

            (
                Box::new(transcode::resampler::Resampler::new(
                    net_channels,
                    audio_channels,
                    net_sample_rate,
                    audio_sample_rate,
                    transcoder_reader,
                    transcoder_writer,
                )) as DynTranscoder,
                net_writer,
                audio_reader,
            )
        }
    };

    let mut encoder: Box<dyn codec::Encoder<f32, _> + Send>;
    let mut decoder: Box<dyn codec::Decoder<f32, _> + Send>;

    match codec_to_use {
        CodecToUse::Opus => {
            let opus_encoder_buf: Box<[f32]> = buffer(codec::opus::buf_size(
                net_capture_stream_config.sample_rate(),
                net_capture_stream_config.channels(),
                codec::opus::SupportedFrameSizeMS::F20,
                false,
            ));
            let opus_decoder_buf: Box<[f32]> = buffer(codec::opus::buf_size(
                net_playback_stream_config.sample_rate(),
                net_playback_stream_config.channels(),
                codec::opus::SupportedFrameSizeMS::F20,
                false,
            ));

            encoder = Box::new(codec::opus::make_encoder(
                net_capture_stream_config,
                opus_encoder_buf,
            )?);
            decoder = Box::new(codec::opus::make_decoder(
                net_playback_stream_config,
                opus_decoder_buf,
            )?);
        }
        CodecToUse::Raw => {
            encoder = Box::new(codec::raw::Encoder);
            decoder = Box::new(codec::raw::Decoder);
        }
    };

    let audio_backend = continuation(capture_data_writer, playback_data_reader)?;
    run_audio_backend(audio_backend);

    let mut transcode_service = transcode_service::TranscodeService {
        capture_transcoder,
        playback_transcoder,
    };

    let mut net_service = net::NetService {
        send_service: net::SendService {
            capture_sample: PhantomData,
            capture_data_reader,
            encoder: &mut *encoder,
            stats: net::SendStats::default(),
        },
        recv_service: net::RecvService {
            playback_sample: PhantomData,
            playback_data_writer,
            decoder: &mut *decoder,
            stats: net::RecvStats::default(),
        },
    };

    rt.block_on(select_first(
        net_service
            .net_loop(socket, send_addrs)
            .with_logger(logger().new(o!("logger" => "net")))
            .boxed(),
        transcode_service
            .transcode_loop()
            .with_logger(logger().new(o!("logger" => "transcode")))
            .boxed(),
    ))?;

    unreachable!();
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

fn run_audio_backend(audio_backend: Box<dyn Backend + 'static>) {
    std::thread::spawn(move || {
        let mut local = audio_backend;
        futures::executor::block_on(local.run())
    });
}

fn buffer<T: Default + Clone>(size: usize) -> Box<[T]> {
    let mut vec = Vec::with_capacity(size);
    let cap = vec.capacity();
    vec.resize(cap, T::default());
    vec.into()
}
