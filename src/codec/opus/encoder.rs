use super::Error;
use crate::io::{ReadSamples, SamplesAvailable};
use audiopus::coder::Encoder as OpusEncoder;

#[derive(Debug)]
pub struct Encoder {
    pub(super) opus: OpusEncoder,
    pub(super) buf: Box<[f32]>,
}

impl Encoder {
    pub fn encode_float<T>(&mut self, input: &mut T, output: &mut [u8]) -> Result<usize, Error>
    where
        T: ReadSamples<f32> + SamplesAvailable,
    {
        let samples_available = input.samples_available()?;
        let samples_required = self.buf.len();
        if samples_available < samples_required {
            return Err(Error::NotEnoughData {
                samples_available,
                samples_required,
            });
        }

        let samples_read = input.read_samples(&mut self.buf)?;
        assert_eq!(samples_read, samples_required);

        let bytes_written = self.opus.encode_float(&self.buf, output)?;
        Ok(bytes_written)
    }
}

impl<T: ReadSamples<f32> + SamplesAvailable> super::super::Encoder<f32, T> for Encoder {
    fn encode(
        &mut self,
        input: &mut T,
        output: &mut [u8],
    ) -> Result<usize, super::super::EncodingError> {
        self.encode_float(input, output).map_err(|err| err.into())
    }
}
