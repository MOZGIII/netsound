use super::Error;
use crate::samples::Samples;
use audiopus::coder::Decoder as OpusDecoder;

#[derive(Debug)]
pub struct Decoder<'a> {
    pub(super) opus: OpusDecoder,
    pub(super) buf: &'a mut [f32],
    pub(super) fec: bool,
    pub(super) channels: usize,
}

impl<'a> Decoder<'a> {
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

impl<'a> super::super::Decoder for Decoder<'a> {
    fn decode(
        &mut self,
        input: &[u8],
        output: &mut Samples,
    ) -> Result<usize, super::super::DecodingError> {
        self.decode_float(input, output, self.fec)
            .map_err(|err| err.into())
    }
}
