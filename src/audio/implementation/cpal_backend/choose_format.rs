use super::{format, CompatibleSample};
use crate::format::Format;

pub fn choose_format<S: CompatibleSample, I: Iterator<Item = cpal::SupportedFormat>>(
    iter: I,
    requested_formats: &[Format<S>],
) -> Result<Format<S>, super::errors::Error> {
    let supported_formats: Vec<_> = iter.collect();
    let cpal_sample_format = S::cpal_sample_format();

    // Try to use format from the preferred formats list.
    for requested_format in requested_formats {
        for supported_format in &supported_formats {
            if supported_format.data_type != cpal_sample_format {
                continue;
            }
            if supported_format.channels != requested_format.channels {
                continue;
            }
            if requested_format.sample_rate < supported_format.min_sample_rate.0
                || requested_format.sample_rate > supported_format.max_sample_rate.0
            {
                continue;
            }

            return Ok(*requested_format);
        }
    }

    // Preferred format wasn't found, use the first one that's supported.
    if let Some(format) = supported_formats.into_iter().next() {
        return Ok(format::from_cpal_supported_format(format));
    }

    Err(super::errors::Error::FormatNegotiationError)
}
