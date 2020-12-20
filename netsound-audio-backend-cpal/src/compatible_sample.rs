use netsound_core::sample::Sample;

pub trait CompatibleSample: Sample {
    fn unwrap_cpal_input_buffer<'a>(buf: &'a cpal::UnknownTypeInputBuffer<'a>) -> &'a [Self];
    fn unwrap_cpal_output_buffer<'a>(
        buf: &'a mut cpal::UnknownTypeOutputBuffer<'a>,
    ) -> &'a mut [Self];
    fn cpal_sample_format() -> cpal::SampleFormat;
}

fn unexpected_buffer_type() -> ! {
    panic!("Unexpected buffer type")
}

macro_rules! impl_compatibe_sample {
    ($([$sample_type:ty, $sample_buffer_variant:ident, $sample_format_variant:ident])*) => {
        $(
            impl CompatibleSample for $sample_type {
                fn unwrap_cpal_input_buffer<'a>(
                    buf: &'a cpal::UnknownTypeInputBuffer<'a>
                ) -> &'a [Self] {
                    match buf {
                        cpal::UnknownTypeInputBuffer::$sample_buffer_variant(buffer) => &*buffer,
                        _ => unexpected_buffer_type(),
                    }
                }

                fn unwrap_cpal_output_buffer<'a>(
                    buf: &'a mut cpal::UnknownTypeOutputBuffer<'a>,
                ) -> &'a mut [Self] {
                    match buf {
                        cpal::UnknownTypeOutputBuffer::$sample_buffer_variant(buffer) => &mut *buffer,
                        _ => unexpected_buffer_type(),
                    }
                }

                fn cpal_sample_format() -> cpal::SampleFormat {
                    cpal::SampleFormat::$sample_format_variant
                }
            }
        )*
    };
}

impl_compatibe_sample![
    [u16, U16, U16]
    [i16, I16, I16]
    [f32, F32, F32]
];

pub mod format {
    use super::CompatibleSample;
    use netsound_core::format::Format;

    fn sample_formats_do_not_match() -> ! {
        panic!("sample formats do not match")
    }

    #[allow(clippy::module_name_repetitions)]
    #[allow(dead_code)]
    pub fn from_cpal_format<S: CompatibleSample>(f: &cpal::Format) -> Format<S> {
        if S::cpal_sample_format() != f.data_type {
            sample_formats_do_not_match();
        }
        Format::<S>::new(f.channels, f.sample_rate.0)
    }

    #[allow(clippy::module_name_repetitions)]
    #[allow(dead_code)]
    pub fn from_cpal_supported_format<S: CompatibleSample>(f: &cpal::SupportedFormat) -> Format<S> {
        if S::cpal_sample_format() != f.data_type {
            sample_formats_do_not_match();
        }
        Format::<S>::new(f.channels, f.max_sample_rate.0)
    }

    #[allow(clippy::module_name_repetitions)]
    #[allow(dead_code)]
    pub fn to_cpal_format<S: CompatibleSample>(f: Format<S>) -> cpal::Format {
        cpal::Format {
            channels: f.channels,
            sample_rate: cpal::SampleRate(f.sample_rate),
            data_type: S::cpal_sample_format(),
        }
    }
}
