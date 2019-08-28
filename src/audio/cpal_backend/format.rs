pub fn choose<I: Iterator<Item = cpal::SupportedFormat>>(
    iter: I,
    requested_formats: &[crate::audio::Format],
) -> Result<cpal::Format, super::errors::Error> {
    let supported_formats: Vec<_> = iter.collect();

    // FIXME: Windows hack.
    if supported_formats.len() == 1 {
        if let cpal::SupportedFormat {
            min_sample_rate: cpal::SampleRate(48000),
            max_sample_rate: cpal::SampleRate(48000),
            data_type: cpal::SampleFormat::F32,
            channels: 2,
        } = &supported_formats[0]
        {
            return Ok((&requested_formats[0]).into());
        }
    }

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

    Err(super::errors::Error::FormatNegotiationError)
}
