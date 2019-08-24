use super::Error;
use crate::samples::Samples;
use audiopus::coder::Decoder as OpusDecoder;

#[derive(Debug)]
pub struct Decoder {
    pub(super) opus: OpusDecoder,
    pub(super) buf: Box<[f32]>,
    pub(super) fec: bool,
    pub(super) channels: usize,
}

impl Decoder {
    pub fn decode_float(
        &mut self,
        input: &[u8],
        output: &mut Samples,
        fec: bool,
    ) -> Result<usize, Error> {
        let audiosize = {
            let buf = &mut self.buf[..];
            self.opus.decode_float(input, buf, fec)?
        };
        let bufsize = audiosize * self.channels;
        let size = output.write_f32(&self.buf[..bufsize]);
        Ok(size)
    }
}

impl super::super::Decoder for Decoder {
    fn decode(
        &mut self,
        input: &[u8],
        output: &mut Samples,
    ) -> Result<usize, super::super::DecodingError> {
        self.decode_float(input, output, self.fec)
            .map_err(|err| err.into())
    }
}
