//! An opus codec implementation.

#![warn(rust_2018_idioms, missing_debug_implementations, missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

mod common;
mod decoder;
mod encoder;
pub mod error;
mod meta;

pub use decoder::Decoder;
pub use encoder::Encoder;

pub use meta::*;
