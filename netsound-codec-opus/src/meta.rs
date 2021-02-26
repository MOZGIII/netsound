use netsound_core::pcm;

/// Compute the required buffer size *in samples count* to accomodate
/// the raw PCM samples under the specified parameters.
#[must_use]
pub fn compute_required_buf_size(
    channels: pcm::Channels,
    samples_per_second: pcm::SampleRate,
) -> usize {
    // 120 - max amount of audio data in milliseconds in a single opus packet
    // We divide by 1000 to account for samples_per_second to ms conversion.
    channels * samples_per_second.as_usize() * 120 / 1000
}
