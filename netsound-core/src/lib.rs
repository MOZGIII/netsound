#![warn(rust_2018_idioms, missing_debug_implementations)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

#[macro_use]
extern crate derivative;

pub use anyhow::Error;

pub mod audio_backend;
pub mod buf;
pub mod codec;
pub mod future;
pub mod io;
pub mod log;
pub mod match_channels;
pub mod net;
pub mod pcm;
pub mod samples_filter;
pub mod transcode;
pub mod transcode_service;
