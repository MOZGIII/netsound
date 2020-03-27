use crate::sample::Sample;
use libpulse_binding::sample as pulse_sample;

pub trait CompatibleSample: Sample {
    fn pulse_sample_format() -> pulse_sample::Format;
}

macro_rules! impl_compatibe_sample {
    ($([$sample_type:ty, $sample_format:ident])*) => {
        $(
            impl CompatibleSample for $sample_type {
                fn pulse_sample_format() -> pulse_sample::Format {
                    pulse_sample::$sample_format
                }
            }
        )*
    };
}

impl_compatibe_sample![[f32, SAMPLE_FLOAT32]];

pub mod format {
    use super::{pulse_sample, CompatibleSample};
    use crate::format::Format;

    fn sample_formats_do_not_match() -> ! {
        panic!("sample formats do not match")
    }

    #[allow(dead_code)]
    pub fn from_pulse_sample_spec<S: CompatibleSample>(spec: pulse_sample::Spec) -> Format<S> {
        if S::pulse_sample_format() != spec.format {
            sample_formats_do_not_match();
        }
        Format::<S>::new(spec.channels.into(), spec.rate)
    }

    #[allow(dead_code)]
    pub fn to_pulse_sample_spec<S: CompatibleSample>(f: Format<S>) -> pulse_sample::Spec {
        use std::convert::TryInto;
        pulse_sample::Spec {
            channels: f
                .channels
                .try_into()
                .expect("unsupported amount of channels"),
            rate: f.sample_rate,
            format: S::pulse_sample_format(),
        }
    }
}
