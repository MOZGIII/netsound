use super::cpal;

pub fn choose<T: Iterator<Item = cpal::SupportedFormat>>(
    iter: &mut T,
    requested_formats: &[crate::audio::Format],
) -> Result<cpal::Format, super::errors::Error> {
    let supported_formats: Vec<_> = iter.collect();

    for requested_format in requested_formats {
        for supported_format in &supported_formats {
            if supported_format.data_type != cpal::SampleFormat::F32 {
                continue;
            }
            if supported_format.channels != requested_format.channels {
                continue;
            }

            if requested_format.sample_rate >= supported_format.min_sample_rate.0
                && requested_format.sample_rate <= supported_format.max_sample_rate.0
            {
                return Ok(requested_format.into());
            }
        }
    }

    Err(super::errors::Error::FormatNegotioationError)
}
