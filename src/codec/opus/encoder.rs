use super::Error;
use crate::samples::Samples;
use audiopus::coder::Encoder as OpusEncoder;

pub struct Encoder<'a> {
    pub opus: OpusEncoder,
    pub buf: &'a mut [f32],
}

impl<'a> Encoder<'a> {
    pub fn encode_float(&mut self, input: &mut Samples, output: &mut [u8]) -> Result<usize, Error> {
        if input.len() < self.buf.len() {
            return Ok(0);
        }

        let size = input.read_f32(self.buf);
        let opus_buf = &self.buf[..size];
        // dbg!(opus_buf.len());
        let size = self.opus.encode_float(opus_buf, output)?;
        Ok(size)
    }
}

impl<'a> super::super::Encoder for Encoder<'a> {
    fn encode(
        &mut self,
        input: &mut Samples,
        output: &mut [u8],
    ) -> Result<usize, super::super::EncodingError> {
        self.encode_float(input, output).map_err(|err| err.into())
    }
}