use super::{choose_format::choose_format, *};
use crate::audio;
use crate::format::Format;
use crate::io::{AsyncReadItems, AsyncWriteItems};
use crate::log::*;
use std::marker::PhantomData;

use cpal::traits::*;

#[derive(Debug)]
pub struct FormatNegotiator;

impl<TCaptureSample, TPlaybackSample> audio::FormatNegotiator<TCaptureSample, TPlaybackSample>
    for FormatNegotiator
where
    TCaptureSample: CompatibleSample,
    TPlaybackSample: CompatibleSample,
{
    type Continuation = FormatNegotiationContinuation<TCaptureSample, TPlaybackSample>;

    fn negotiate_formats<'a>(
        self,
        request_capture_formats: &'a [Format<TCaptureSample>],
        request_playback_formats: &'a [Format<TPlaybackSample>],
        logger: Logger,
    ) -> Result<
        (
            audio::NegotiatedFormats<TCaptureSample, TPlaybackSample>,
            Self::Continuation,
        ),
        crate::Error,
    > {
        let cpal_host = cpal::default_host();
        info!(logger, "Cpal Host: {:?}", &cpal_host.id());

        let cpal_event_loop = cpal_host.event_loop();

        let cpal_input_device = default::input_device(&cpal_host)?;
        let cpal_output_device = default::output_device(&cpal_host)?;

        let capture_format = choose_format(
            cpal_input_device.supported_input_formats()?,
            request_capture_formats,
        )?;
        let playback_format = choose_format(
            cpal_output_device.supported_output_formats()?,
            request_playback_formats,
        )?;

        let negotiated_formats = audio::NegotiatedFormats {
            capture_format,
            playback_format,
        };

        let continuation = FormatNegotiationContinuation {
            cpal_event_loop,
            cpal_input_device,
            cpal_output_device,
            capture_format,
            playback_format,
            logger,
        };

        Ok((negotiated_formats, continuation))
    }
}

pub struct FormatNegotiationContinuation<TCaptureSample, TPlaybackSample>
where
    TCaptureSample: CompatibleSample,
    TPlaybackSample: CompatibleSample,
{
    cpal_event_loop: <cpal::Host as HostTrait>::EventLoop,

    cpal_input_device: <cpal::Host as HostTrait>::Device,
    cpal_output_device: <cpal::Host as HostTrait>::Device,

    capture_format: Format<TCaptureSample>,
    playback_format: Format<TPlaybackSample>,

    logger: Logger,
}

pub struct BackendBuilder<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>
where
    TCaptureSample: CompatibleSample + Send + Sync,
    TPlaybackSample: CompatibleSample + Send + Sync,

    TCaptureDataWriter: AsyncWriteItems<TCaptureSample> + Unpin + Send + Sync,
    TPlaybackDataReader: AsyncReadItems<TPlaybackSample> + Unpin + Send + Sync,
{
    pub continuation: FormatNegotiationContinuation<TCaptureSample, TPlaybackSample>,

    pub capture_data_writer: TCaptureDataWriter,
    pub playback_data_reader: TPlaybackDataReader,
}

impl<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader> audio::BackendBuilder
    for BackendBuilder<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>
where
    TCaptureSample: CompatibleSample + Send + Sync,
    TPlaybackSample: CompatibleSample + Send + Sync,

    TCaptureDataWriter: AsyncWriteItems<TCaptureSample> + Unpin + Send + Sync,
    TPlaybackDataReader: AsyncReadItems<TPlaybackSample> + Unpin + Send + Sync,
{
    type Backend =
        Backend<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>;

    fn build(self) -> Result<Self::Backend, crate::Error> {
        let cpal_capture_format = format::to_cpal_format(self.continuation.capture_format);
        let cpal_playback_format = format::to_cpal_format(self.continuation.playback_format);
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
