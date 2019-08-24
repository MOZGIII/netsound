use crate::samples::Samples;

mod codec;

pub type Endian = byteorder::LittleEndian;

#[derive(Debug)]
pub struct Encoder;

impl super::Encoder for Encoder {
    fn encode(
        &mut self,
        input: &mut Samples,
        output: &mut [u8],
    ) -> Result<usize, super::EncodingError> {
        Ok(codec::encode::<Endian>(input, output))
    }
}

#[derive(Debug)]
pub struct Decoder;

impl super::Decoder for Decoder {
    fn decode(
        &mut self,
        input: &[u8],
        output: &mut Samples,
    ) -> Result<usize, super::DecodingError> {
        Ok(codec::decode::<Endian>(input, output))
    }
}
