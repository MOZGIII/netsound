#![warn(rust_2018_idioms, missing_debug_implementations)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub use failure::Error;

pub mod audio;
pub mod buf;
pub mod codec;
pub mod format;
pub mod formats;
pub mod future;
pub mod io;
pub mod log;
pub mod match_channels;
pub mod net;
pub mod sample;
pub mod sample_type_name;
pub mod samples_filter;
pub mod transcode;
pub mod transcode_service;
