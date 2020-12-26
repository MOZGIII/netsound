use std::net::SocketAddr;

use structopt::StructOpt;

#[derive(StructOpt)]
pub enum Command {
    /// Run the app.
    Run(RunParams),
    /// List available audio backends.
    ListAudioBackends,
}

#[derive(StructOpt)]
pub struct RunParams {
    /// Interface address and the port to bind to.
    #[structopt(short = "b", long = "bind", default_value = "127.0.0.1:8080")]
    pub bind_addr: SocketAddr,
    /// A list of host:port pairs to send the audio packets to.
    /// If not set, data is sent to the binded address (loopback).
    pub send_addrs: Vec<SocketAddr>,
}
