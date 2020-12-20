use netsound_core::pcm::Sample;

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

pub mod stream_config {
    use std::convert::TryInto;

    use super::CompatibleSample;
    use netsound_core::pcm::StreamConfig;

    fn sample_formats_do_not_match() -> ! {
        panic!("sample formats do not match")
    }

    #[must_use]
    pub fn from_cpal_format<S: CompatibleSample>(f: &cpal::Format) -> StreamConfig<S> {
        if S::cpal_sample_format() != f.data_type {
            sample_formats_do_not_match();
        }
        let sample_rate: usize = f.sample_rate.0.try_into().unwrap();
        StreamConfig::<S>::new(sample_rate.into(), f.channels.try_into().unwrap())
    }

    #[must_use]
    pub fn from_cpal_supported_format<S: CompatibleSample>(
        sf: &cpal::SupportedFormat,
    ) -> StreamConfig<S> {
        if S::cpal_sample_format() != sf.data_type {
            sample_formats_do_not_match();
        }
        let sample_rate: usize = sf.max_sample_rate.0.try_into().unwrap();
        StreamConfig::<S>::new(sample_rate.into(), sf.channels.into())
    }

    #[must_use]
    pub fn to_cpal_format<S: CompatibleSample>(sc: StreamConfig<S>) -> cpal::Format {
        cpal::Format {
            channels: sc.channels().try_into().unwrap(),
            sample_rate: cpal::SampleRate(sc.sample_rate().as_usize().try_into().unwrap()),
            data_type: S::cpal_sample_format(),
        }
    }
}
