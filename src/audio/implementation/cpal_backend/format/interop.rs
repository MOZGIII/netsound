use super::*;
use crate::format::Format;
use sample::Sample;

use assert::CpalSampleFormatAsserter;
use deduce::CpalSampleFormatDeducer;

#[allow(dead_code)]
pub fn from_cpal_format<S: Sample>(f: cpal::Format) -> Format<S>
where
    assert::ExactCpalSampleFormatAsserter<S>: assert::CpalSampleFormatAsserter<Sample = S>,
{
    let asserter = assert::ExactCpalSampleFormatAsserter::<S>::new();
    asserter.assert(f.data_type);
    Format::<S>::new(f.channels, f.sample_rate.0)
}

#[allow(dead_code)]
pub fn to_cpal_format<S: Sample>(f: Format<S>) -> cpal::Format
where
    deduce::ExactCpalSampleFormatDeducer<S>: deduce::CpalSampleFormatDeducer<Sample = S>,
{
    let deducer = deduce::ExactCpalSampleFormatDeducer::<S>::new();
    cpal::Format {
        channels: f.channels,
        sample_rate: cpal::SampleRate(f.sample_rate),
        data_type: deducer.deduce(),
    }
}
