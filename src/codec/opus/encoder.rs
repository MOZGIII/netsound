use super::Error;
use crate::samples::Samples;
use audiopus::coder::Encoder as OpusEncoder;

#[derive(Debug)]
pub struct Encoder {
    pub(super) opus: OpusEncoder,
    pub(super) buf: Box<[f32]>,
}

impl Encoder {
    pub fn encode_float(&mut self, input: &mut Samples, output: &mut [u8]) -> Result<usize, Error> {
        let bytes_available = input.len();
        let bytes_required = self.buf.len();
        if bytes_available < bytes_required {
            return Err(Error::NotEnoughData {
                bytes_available,
                bytes_required,
            });
        }

        let size = input.read_f32(&mut self.buf);
        let opus_buf = &self.buf[..size];
        // dbg!(opus_buf.len());
        let size = self.opus.encode_float(opus_buf, output)?;
        Ok(size)
    }
}

impl super::super::Encoder for Encoder {
    fn encode(
        &mut self,
        input: &mut Samples,
        output: &mut [u8],
    ) -> Result<usize, super::super::EncodingError> {
        self.encode_float(input, output).map_err(|err| err.into())
    }
}
