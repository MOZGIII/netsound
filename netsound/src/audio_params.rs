use super::pcm;

static AUDIO_PARAMS: &[(pcm::SampleRate, pcm::Channels)] = &[
    (pcm::SampleRate::from_usize(48000), 2),
    (pcm::SampleRate::from_usize(48000), 1),
    (pcm::SampleRate::from_usize(24000), 2),
    (pcm::SampleRate::from_usize(24000), 1),
    (pcm::SampleRate::from_usize(16000), 2),
    (pcm::SampleRate::from_usize(16000), 1),
    (pcm::SampleRate::from_usize(12000), 2),
    (pcm::SampleRate::from_usize(12000), 1),
    (pcm::SampleRate::from_usize(8000), 2),
    (pcm::SampleRate::from_usize(8000), 1),
];

pub fn input() -> &'static [(pcm::SampleRate, pcm::Channels)] {
    AUDIO_PARAMS
}

pub fn output() -> &'static [(pcm::SampleRate, pcm::Channels)] {
    AUDIO_PARAMS
}
