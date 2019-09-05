use crate::io::{ReadItems, WriteItems};
use sample::Sample;

pub mod opus;
pub mod raw;

mod error;
pub use self::error::{DecodingError, EncodingError};

pub trait Encoder<S: Sample, T: ReadItems<S>> {
    fn encode(&mut self, input: &mut T, output: &mut [u8]) -> Result<usize, EncodingError>;
}

pub trait Decoder<S: Sample, T: WriteItems<S>> {
    fn decode(&mut self, input: &[u8], output: &mut T) -> Result<usize, DecodingError>;
}
