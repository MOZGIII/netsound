use crate::{control, io};

use super::{
    choose_stream_config::choose_stream_config, default, stream_config, Backend, CompatibleSample,
};
use futures::{executor::block_on, SinkExt};
use netsound_core::io::{AsyncReadItems, AsyncWriteItems};
use netsound_core::log::no_scopes::{info, slog_info, Logger};
use netsound_core::pcm::StreamConfig;
use netsound_core::{audio_backend, log::no_scopes::trace};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

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

        let cpal_input_device = default::input_device(&cpal_host)?;
        let cpal_output_device = default::output_device(&cpal_host)?;

        let capture_stream_config = choose_stream_config(
            &mut logger,
            cpal_input_device.supported_input_configs()?,
            requested_capture_stream_configs,
        )?;
        let playback_stream_config = choose_stream_config(
            &mut logger,
            cpal_output_device.supported_output_configs()?,
            requested_playback_stream_configs,
        )?;

        let negotiated_stream_configs = audio_backend::NegotiatedStreamConfigs {
            capture: capture_stream_config,
            playback: playback_stream_config,
        };

        let continuation = StreamConfigNegotiationContinuation {
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
    TCaptureSample: CompatibleSample + Send + Sync + 'static,
    TPlaybackSample: CompatibleSample + Send + Sync + 'static,

    TCaptureDataWriter: AsyncWriteItems<TCaptureSample> + Unpin + Send + Sync + 'static,
    TPlaybackDataReader: AsyncReadItems<TPlaybackSample> + Unpin + Send + Sync + 'static,
{
    type Backend = Backend;

    fn build(self) -> Result<Self::Backend, netsound_core::Error> {
        let cpal_capture_stream_config =
            stream_config::to_cpal(self.continuation.capture_stream_config);
        let cpal_playback_stream_config =
            stream_config::to_cpal(self.continuation.playback_stream_config);
        let mut logger = self.continuation.logger;

        log_config(
            &mut logger,
            "Playback",
            &self.continuation.cpal_output_device.name()?,
            &cpal_playback_stream_config,
        );
        log_config(
            &mut logger,
            "Capture",
            &self.continuation.cpal_input_device.name()?,
            &cpal_capture_stream_config,
        );

        let (error_tx, error_rx) = control::channel(0);
        let (drop_tx, drop_rx) = futures::channel::oneshot::channel::<()>();

        let cpal_output_device = self.continuation.cpal_output_device;
        let cpal_input_device = self.continuation.cpal_input_device;

        let playback_data_reader = self.playback_data_reader;
        let capture_data_writer = self.capture_data_writer;

        let logger_clone = logger.clone();

        std::thread::spawn(move || {
            let logger = logger_clone;

            let mut playback_data_reader = playback_data_reader;
            let mut capture_data_writer = capture_data_writer;

            let logger_clone = logger.clone();
            let mut error_tx_clone = error_tx.clone();
            let cpal_output_stream = cpal_output_device
                .build_output_stream(
                    &cpal_playback_stream_config,
                    move |data: &mut [TPlaybackSample], _: &cpal::OutputCallbackInfo| {
                        trace!(logger_clone, "cpal: before play");
                        io::play(&mut playback_data_reader, data);
                        trace!(logger_clone, "cpal: after play");
                    },
                    move |err| {
                        block_on(async { error_tx_clone.send(("playback", err)).await.unwrap() })
                    },
                )
                .unwrap();

            let logger_clone = logger.clone();
            let mut error_tx_clone = error_tx.clone();
            let cpal_input_stream = cpal_input_device
                .build_input_stream(
                    &cpal_capture_stream_config,
                    move |data: &[TCaptureSample], _: &cpal::InputCallbackInfo| {
                        trace!(logger_clone, "cpal: before capture");
                        io::capture(data, &mut capture_data_writer);
                        trace!(logger_clone, "cpal: after capture");
                    },
                    move |err| {
                        block_on(async { error_tx_clone.send(("capture", err)).await.unwrap() })
                    },
                )
                .unwrap();

            cpal_output_stream.play().unwrap();
            cpal_input_stream.play().unwrap();

            futures::executor::block_on(async {
                let _ = drop_rx.await;
            });

            cpal_output_stream.pause().unwrap();
            cpal_input_stream.pause().unwrap();

            drop(cpal_output_stream);
            drop(cpal_input_stream);
        });

        let backend = Backend {
            error_rx,
            drop_tx,
            logger,
        };
        Ok(backend)
    }
}

fn log_config(
    logger: &mut Logger,
    name: &'static str,
    device_name: &str,
    stream_config: &cpal::StreamConfig,
) {
    slog_info!(logger, "{} device: {}", name, device_name);
    slog_info!(logger, "{} stream config: {:?}", name, stream_config);
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
