use super::{
    choose_stream_config::choose_stream_config, default, stream_config, Backend, CompatibleSample,
};
use netsound_core::audio_backend;
use netsound_core::io::{AsyncReadItems, AsyncWriteItems};
use netsound_core::log::no_scopes::{info, slog_info, Logger};
use netsound_core::pcm::StreamConfig;
use std::marker::PhantomData;

use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};

#[derive(Debug)]
pub struct StreamConfigNegotiator;

impl<TCaptureSample, TPlaybackSample>
    audio_backend::StreamConfigNegotiator<TCaptureSample, TPlaybackSample>
    for StreamConfigNegotiator
where
    TCaptureSample: CompatibleSample,
    TPlaybackSample: CompatibleSample,
{
    type Continuation = StreamConfigNegotiationContinuation<TCaptureSample, TPlaybackSample>;

    fn negotiate<'a>(
        self,
        requested_capture_stream_configs: &'a [StreamConfig<TCaptureSample>],
        requested_playback_stream_configs: &'a [StreamConfig<TPlaybackSample>],
        mut logger: Logger,
    ) -> Result<
        (
            audio_backend::NegotiatedStreamConfigs<TCaptureSample, TPlaybackSample>,
            Self::Continuation,
        ),
        netsound_core::Error,
    > {
        let cpal_host = cpal::default_host();
        info!(logger, "Cpal Host: {:?}", &cpal_host.id());

        let cpal_event_loop = cpal_host.event_loop();

        let cpal_input_device = default::input_device(&cpal_host)?;
        let cpal_output_device = default::output_device(&cpal_host)?;

        let capture_stream_config = choose_stream_config(
            &mut logger,
            cpal_input_device.supported_input_formats()?,
            requested_capture_stream_configs,
        )?;
        let playback_stream_config = choose_stream_config(
            &mut logger,
            cpal_output_device.supported_output_formats()?,
            requested_playback_stream_configs,
        )?;

        let negotiated_stream_configs = audio_backend::NegotiatedStreamConfigs {
            capture: capture_stream_config,
            playback: playback_stream_config,
        };

        let continuation = StreamConfigNegotiationContinuation {
            cpal_event_loop,
            cpal_input_device,
            cpal_output_device,
            capture_stream_config,
            playback_stream_config,
            logger,
        };

        Ok((negotiated_stream_configs, continuation))
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct StreamConfigNegotiationContinuation<TCaptureSample, TPlaybackSample>
where
    TCaptureSample: CompatibleSample,
    TPlaybackSample: CompatibleSample,
{
    #[derivative(Debug = "ignore")]
    cpal_event_loop: <cpal::Host as HostTrait>::EventLoop,

    #[derivative(Debug = "ignore")]
    cpal_input_device: <cpal::Host as HostTrait>::Device,
    #[derivative(Debug = "ignore")]
    cpal_output_device: <cpal::Host as HostTrait>::Device,

    capture_stream_config: StreamConfig<TCaptureSample>,
    playback_stream_config: StreamConfig<TPlaybackSample>,

    logger: Logger,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct BackendBuilder<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>
where
    TCaptureSample: CompatibleSample + Send + Sync,
    TPlaybackSample: CompatibleSample + Send + Sync,

    TCaptureDataWriter: AsyncWriteItems<TCaptureSample> + Unpin + Send + Sync,
    TPlaybackDataReader: AsyncReadItems<TPlaybackSample> + Unpin + Send + Sync,
{
    pub continuation: StreamConfigNegotiationContinuation<TCaptureSample, TPlaybackSample>,

    pub capture_data_writer: TCaptureDataWriter,
    pub playback_data_reader: TPlaybackDataReader,
}

impl<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>
    audio_backend::Builder
    for BackendBuilder<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>
where
    TCaptureSample: CompatibleSample + Send + Sync,
    TPlaybackSample: CompatibleSample + Send + Sync,

    TCaptureDataWriter: AsyncWriteItems<TCaptureSample> + Unpin + Send + Sync,
    TPlaybackDataReader: AsyncReadItems<TPlaybackSample> + Unpin + Send + Sync,
{
    type Backend =
        Backend<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>;

    fn build(self) -> Result<Self::Backend, netsound_core::Error> {
        let cpal_capture_format =
            stream_config::to_cpal_format(self.continuation.capture_stream_config);
        let cpal_playback_format =
            stream_config::to_cpal_format(self.continuation.playback_stream_config);
        let mut logger = self.continuation.logger;

        log_config(
            &mut logger,
            "Playback",
            &self.continuation.cpal_output_device.name()?,
            &cpal_playback_format,
        );
        log_config(
            &mut logger,
            "Capture",
            &self.continuation.cpal_input_device.name()?,
            &cpal_capture_format,
        );

        let cpal_event_loop = self.continuation.cpal_event_loop;

        let playback_stream_id = cpal_event_loop
            .build_output_stream(&self.continuation.cpal_output_device, &cpal_playback_format)?;
        let capture_stream_id = cpal_event_loop
            .build_input_stream(&self.continuation.cpal_input_device, &cpal_capture_format)?;

        cpal_event_loop.play_stream(playback_stream_id.clone())?;
        cpal_event_loop.play_stream(capture_stream_id.clone())?;

        let backend = Backend {
            capture_sample: PhantomData,
            playback_sample: PhantomData,

            capture_data_writer: self.capture_data_writer,
            playback_data_reader: self.playback_data_reader,

            capture_stream_id,
            playback_stream_id,

            cpal_event_loop,

            logger,
        };
        Ok(backend)
    }
}

fn log_config(logger: &mut Logger, name: &'static str, device_name: &str, format: &cpal::Format) {
    slog_info!(logger, "{} device: {}", name, device_name);
    slog_info!(logger, "{} format: {:?}", name, format);
    slog_info!(
        logger,
        "{} endianness: {}",
        name,
        if cfg!(target_endian = "little") {
            "little"
        } else {
            "big"
        }
    );
    // Always interleaved.
    slog_info!(logger, "{} operation mode: interleaved", name);
}
