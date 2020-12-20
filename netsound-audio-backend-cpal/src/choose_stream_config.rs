use std::convert::TryInto;

use super::{stream_config, CompatibleSample};
use netsound_core::log::no_scopes::{trace, Logger};
use netsound_core::pcm::StreamConfig;

pub fn choose_stream_config<S: CompatibleSample, I: Iterator<Item = cpal::SupportedFormat>>(
    logger: &mut Logger,
    iter: I,
    requested_stream_configs: &[StreamConfig<S>],
) -> Result<StreamConfig<S>, super::errors::Error> {
    let supported_formats: Vec<_> = iter.collect();
    let cpal_sample_format = S::cpal_sample_format();

    // Try to use stream config from the preferred stream configs list.
    for requested_stream_config in requested_stream_configs {
        trace!(
            logger,
            "Trying requested stream config {:?}",
            requested_stream_config
        );

        let requested_stream_config_cpal_channels: u16 =
            requested_stream_config.channels().try_into().unwrap();
        let requested_stream_config_cpal_sample_rate: u32 = requested_stream_config
            .sample_rate()
            .as_usize()
            .try_into()
            .unwrap();

        for supported_format in &supported_formats {
            trace!(
                logger,
                "Matching with supported format {:?}",
                supported_format
            );
            if supported_format.data_type != cpal_sample_format {
                continue;
            }
            if supported_format.channels != requested_stream_config_cpal_channels {
                continue;
            }
            if !(supported_format.min_sample_rate.0..=supported_format.max_sample_rate.0)
                .contains(&requested_stream_config_cpal_sample_rate)
            {
                continue;
            }

            trace!(
                logger,
                "Matched requested format {:?} with supported format {:?}",
                requested_stream_config,
                supported_format
            );
            return Ok(*requested_stream_config);
        }
    }

    // Preferred format wasn't found, use the first one that's supported.
    if let Some(ref format) = supported_formats.into_iter().next() {
        return Ok(stream_config::from_cpal_supported_format(format));
    }

    Err(super::errors::Error::StreamConfigNegotiation)
}
