use std::convert::TryInto;

use super::{stream_config, CompatibleSample};
use netsound_core::log::no_scopes::{trace, Logger};
use netsound_core::pcm::StreamConfig;

pub fn choose_stream_config<S, I>(
    logger: &mut Logger,
    iter: I,
    requested_stream_configs: &[StreamConfig<S>],
) -> Option<StreamConfig<S>>
where
    S: CompatibleSample,
    I: Iterator<Item = cpal::SupportedStreamConfigRange>,
{
    let supported_ranges: Vec<_> = iter.collect();
    let cpal_sample_format = <S as cpal::Sample>::FORMAT;

    // Try to use stream config from the preferred stream configs list.
    for requested_stream_config in requested_stream_configs {
        trace!(
            logger,
            "Trying requested stream config {:?}",
            requested_stream_config
        );

        let requested_stream_config_cpal_channels: u16 =
            requested_stream_config.channels().try_into().unwrap();
        let requested_stream_config_cpal_sample_rate = cpal::SampleRate(
            requested_stream_config
                .sample_rate()
                .as_usize()
                .try_into()
                .unwrap(),
        );

        for supported_range in &supported_ranges {
            trace!(
                logger,
                "Matching with supported config range {:?}",
                supported_range
            );
            if supported_range.sample_format() != cpal_sample_format {
                continue;
            }
            if supported_range.channels() != requested_stream_config_cpal_channels {
                continue;
            }
            if !(supported_range.min_sample_rate()..=supported_range.max_sample_rate())
                .contains(&requested_stream_config_cpal_sample_rate)
            {
                continue;
            }

            trace!(
                logger,
                "Matched requested format {:?} with supported range {:?}",
                requested_stream_config,
                supported_range
            );
            return Some(*requested_stream_config);
        }
    }

    // Preferred stream config wasn't found, use the first one that's supported.
    if let Some(range) = supported_ranges.into_iter().next() {
        return Some(stream_config::from_cpal_supported(
            &range.with_max_sample_rate(),
        ));
    }

    None
}
