#![warn(rust_2018_idioms, missing_debug_implementations)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

#[macro_use]
extern crate derivative;

mod backend;
mod builder;
mod choose_format;
mod compatible_sample;
mod default;
mod errors;
mod io;

pub use backend::*;
pub use builder::*;
pub use compatible_sample::*;
pub use errors::*;
