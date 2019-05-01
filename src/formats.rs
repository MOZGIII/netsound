use super::audio::Format;

fn formats() -> &'static [Format] {
    &[
        Format {
            channels: 2,
            sample_rate: audiopus::SampleRate::Hz48000 as _,
        },
        Format {
            channels: 1,
            sample_rate: audiopus::SampleRate::Hz48000 as _,
        },
        Format {
            channels: 2,
            sample_rate: audiopus::SampleRate::Hz24000 as _,
        },
        Format {
            channels: 1,
            sample_rate: audiopus::SampleRate::Hz24000 as _,
        },
        Format {
            channels: 2,
            sample_rate: audiopus::SampleRate::Hz16000 as _,
        },
        Format {
            channels: 1,
            sample_rate: audiopus::SampleRate::Hz16000 as _,
        },
        Format {
            channels: 2,
            sample_rate: audiopus::SampleRate::Hz12000 as _,
        },
        Format {
            channels: 1,
            sample_rate: audiopus::SampleRate::Hz12000 as _,
        },
        Format {
            channels: 2,
            sample_rate: audiopus::SampleRate::Hz8000 as _,
        },
        Format {
            channels: 1,
            sample_rate: audiopus::SampleRate::Hz8000 as _,
        },
    ]
}

pub fn input() -> &'static [Format] {
    formats()
}

pub fn output() -> &'static [Format] {
    formats()
}
