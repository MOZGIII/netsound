use crate::io::{ReadItems, WriteItems};

mod codec;

pub type Endian = byteorder::LittleEndian;

#[derive(Debug)]
pub struct Encoder;

impl<T: ReadItems<f32>> super::Encoder<f32, T> for Encoder {
    fn encode(&mut self, input: &mut T, output: &mut [u8]) -> Result<usize, super::EncodingError> {
        Ok(codec::encode::<Endian, T>(input, output)
            .map_err(|err| super::EncodingError::Other(err.into()))?)
    }
}

#[derive(Debug)]
pub struct Decoder;

impl<T: WriteItems<f32>> super::Decoder<f32, T> for Decoder {
    fn decode(&mut self, input: &[u8], output: &mut T) -> Result<usize, super::DecodingError> {
        Ok(codec::decode::<Endian, T>(input, output)
            .map_err(|err| super::DecodingError::Other(err.into()))?)
    }
}
