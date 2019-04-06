extern crate byteorder;
extern crate mio;

use std::env;
use std::net::SocketAddr;

use mio::net::UdpSocket;

mod audio;
mod net;
mod samples;

use audio::Backend;
use audio::BackendBuilderFor;
use samples::Samples;

type BoxedErr = Box<std::error::Error>;

fn main() -> Result<(), BoxedErr> {
    let bind_addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let connect_addr = env::args().nth(2).unwrap_or(bind_addr.clone());
    let bind_addr: SocketAddr = bind_addr.parse()?;
    let connect_addr: SocketAddr = connect_addr.parse()?;

    let socket = UdpSocket::bind(&bind_addr)?;
    println!("Listening on: {}", socket.local_addr()?);

    // TODO: use `socket.peer_addr()` when it lands to stable.
    // https://github.com/rust-lang/rust/issues/59127
    socket.connect(connect_addr.clone())?;
    println!("Connected to: {}", &connect_addr);

    let capture_buf = Samples::shared_with_capacity(30_000_000);
    let playback_buf = Samples::shared_with_capacity(30_000_000);

    let audio_backend_builder = audio::BackendBuilder {
        capture_buf: capture_buf.clone(),
        playback_buf: playback_buf.clone(),
    };
    run_audio_backend(audio_backend_builder)?;

    let net_service = net::NetService {
        capture_buf: capture_buf.clone(),
        playback_buf: playback_buf.clone(),
    };
    net_service.r#loop(socket)?;

    Ok(())
}

#[derive(Debug)]
enum AudioBackendToUse {
    Cpal,
    PulseSimple,
}

impl AudioBackendToUse {
    fn from_env() -> Result<Self, std::env::VarError> {
        Ok(match std::env::var("AUDIO_BACKEND") {
            Ok(ref val) if val == "pulse_simple" => AudioBackendToUse::PulseSimple,
            Ok(ref val) if val == "cpal" => AudioBackendToUse::Cpal,
            // Defaults.
            Ok(_) => AudioBackendToUse::Cpal,
            Err(std::env::VarError::NotPresent) => AudioBackendToUse::Cpal,
            // Invalid value.
            Err(e) => return Err(e),
        })
    }
}

fn run_audio_backend(builder: audio::BackendBuilder) -> Result<(), BoxedErr> {
    let backend_to_use = AudioBackendToUse::from_env()?;
    println!("Using audio backend: {:?}", backend_to_use);

    match backend_to_use {
        AudioBackendToUse::Cpal => run_cpal_backend(builder),
        AudioBackendToUse::PulseSimple => run_pulse_simple_backend(builder),
    }
}

fn run_cpal_backend(builder: audio::BackendBuilder) -> Result<(), BoxedErr> {
    use audio::cpal_backend::Backend as CpalBackend;
    let audio_backend: CpalBackend = builder.build()?;
    std::thread::spawn(move || audio_backend.run());
    return Ok(());
}

#[cfg(feature = "pulse_simple_backend")]
fn run_pulse_simple_backend(builder: audio::BackendBuilder) -> Result<(), BoxedErr> {
    use audio::pulse_simple_backend::Backend as PulseSimpleBackend;
    let audio_backend: PulseSimpleBackend = builder.build()?;
    std::thread::spawn(move || audio_backend.run());
    Ok(())
}

#[cfg(not(feature = "pulse_simple_backend"))]
fn run_pulse_simple_backend(_builder: audio::BackendBuilder) -> Result<(), BoxedErr> {
    unimplemented!();
}
