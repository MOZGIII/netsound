use std::net::SocketAddr;

use structopt::StructOpt;

use crate::{audio_backend_config::AnyAudioBackendVariant, codec_config::CodecToUse};

#[derive(StructOpt)]
pub enum Command {
    /// Run the app.
    Run(RunParams),
    /// List available audio backends.
    ListAudioBackends,
}

#[derive(StructOpt)]
pub struct RunParams {
    /// Audio backend to use.
    #[structopt(
        short = "a",
        long = "audio-backend",
        default_value = "cpal",
        env = "AUDIO_BACKEND"
    )]
    pub audio_backend_variant: AnyAudioBackendVariant,
    /// Audio codec to use.
    #[structopt(short = "c", long = "codec", default_value = "opus", env = "CODEC")]
    pub codec_to_use: CodecToUse,

    /// Interface address and the port to bind to.
    #[structopt(
        short = "b",
        long = "bind",
        default_value = "127.0.0.1:8080",
        env = "BIND_ADDR"
    )]
    pub bind_addr: SocketAddr,
    /// A list of host:port pairs to send the audio packets to.
    /// If not set, data is sent to the binded address (loopback).
    pub send_addrs: Vec<SocketAddr>,
}
