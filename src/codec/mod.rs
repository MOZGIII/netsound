use crate::Samples;

pub mod opus;
pub mod raw;

mod error;
pub use self::error::{DecodingError, EncodingError};

pub trait Encoder {
    fn encode(&mut self, input: &mut Samples, output: &mut [u8]) -> Result<usize, EncodingError>;
}

pub trait Decoder {
    fn decode(&mut self, input: &[u8], output: &mut Samples) -> Result<usize, DecodingError>;
}
