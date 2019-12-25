use super::{format, CompatibleSample};
use crate::format::Format;
use crate::log::no_scopes::*;

pub fn choose_format<S: CompatibleSample, I: Iterator<Item = cpal::SupportedFormat>>(
    logger: &mut Logger,
    iter: I,
    requested_formats: &[Format<S>],
) -> Result<Format<S>, super::errors::Error> {
    let supported_formats: Vec<_> = iter.collect();
    let cpal_sample_format = S::cpal_sample_format();

    // Try to use format from the preferred formats list.
    for requested_format in requested_formats {
        trace!(logger, "Trying requested format {:?}", requested_format);
        for supported_format in &supported_formats {
            trace!(
                logger,
                "Matching with supported format {:?}",
                supported_format
            );
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

            trace!(
                logger,
                "Matched requested format {:?} with supported format {:?}",
                requested_format,
                supported_format
            );
            return Ok(*requested_format);
        }
    }

    // Preferred format wasn't found, use the first one that's supported.
    if let Some(ref format) = supported_formats.into_iter().next() {
        return Ok(format::from_cpal_supported_format(format));
    }

    Err(super::errors::Error::FormatNegotiation)
}
