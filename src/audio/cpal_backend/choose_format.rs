use super::format;
use crate::format::Format;
use sample::Sample;

pub fn choose_format<S: Sample, I: Iterator<Item = cpal::SupportedFormat>>(
    iter: I,
    requested_formats: &[Format<S>],
) -> Result<cpal::Format, super::errors::Error>
where
    format::deduce::ExactCpalSampleFormatDeducer<S>:
        format::deduce::CpalSampleFormatDeducer<Sample = S>,
{
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
            return Ok(format::interop::to_cpal_format(
                requested_formats[0].clone(),
            ));
        }
    }

    use format::deduce::CpalSampleFormatDeducer;
    let deducer = format::deduce::ExactCpalSampleFormatDeducer::<S>::new();
    let cpal_sample_format = deducer.deduce();

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

            return Ok(format::interop::to_cpal_format(requested_format.clone()));
        }
    }

    Err(super::errors::Error::FormatNegotiationError)
}
