use super::Error;
use crate::io::WriteItems;
use audiopus::coder::Decoder as OpusDecoder;

#[derive(Debug)]
pub struct Decoder {
    pub(super) opus: OpusDecoder,
    pub(super) buf: Box<[f32]>,
    pub(super) fec: bool,
    pub(super) channels: usize,
}

impl Decoder {
    pub fn decode_float<T>(
        &mut self,
        input: &[u8],
        output: &mut T,
        fec: bool,
    ) -> Result<usize, Error>
    where
        T: WriteItems<f32>,
    {
        let audiosize = {
            let buf = &mut self.buf[..];
            self.opus.decode_float(input, buf, fec)?
        };
        let bufsize = audiosize * self.channels;
        let size = output.write_items(&self.buf[..bufsize])?;
        Ok(size)
    }
}

impl<T: WriteItems<f32>> super::super::Decoder<f32, T> for Decoder {
    fn decode(
        &mut self,
        input: &[u8],
        output: &mut T,
    ) -> Result<usize, super::super::DecodingError> {
        self.decode_float(input, output, self.fec)
            .map_err(|err| err.into())
    }
}
