use super::Error;
use crate::io::{ItemsAvailable, ReadItems};
use audiopus::coder::Encoder as OpusEncoder;

#[derive(Debug)]
pub struct Encoder {
    pub(super) opus: OpusEncoder,
    pub(super) buf: Box<[f32]>,
}

impl Encoder {
    pub fn encode_float<T>(&mut self, input: &mut T, output: &mut [u8]) -> Result<usize, Error>
    where
        T: ReadItems<f32> + ItemsAvailable<f32>,
    {
        let samples_available = input.items_available()?;
        let samples_required = self.buf.len();
        if samples_available < samples_required {
            return Err(Error::NotEnoughData {
                samples_available,
                samples_required,
            });
        }

        let samples_read = input.read_items(&mut self.buf)?;
        assert_eq!(samples_read, samples_required);

        let bytes_written = self.opus.encode_float(&self.buf, output)?;
        Ok(bytes_written)
    }
}

impl<T: ReadItems<f32> + ItemsAvailable<f32>> super::super::Encoder<f32, T> for Encoder {
    fn encode(
        &mut self,
        input: &mut T,
        output: &mut [u8],
    ) -> Result<usize, super::super::EncodingError> {
        self.encode_float(input, output).map_err(|err| err.into())
    }
}
