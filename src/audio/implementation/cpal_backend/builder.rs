use super::{choose_format::choose_format, *};
use crate::audio;
use crate::format::Format;
use crate::io::{AsyncReadItems, AsyncWriteItems};
use crate::sync::Synced;
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
    ) -> Result<
        (
            audio::NegotiatedFormats<TCaptureSample, TPlaybackSample>,
            Self::Continuation,
        ),
        crate::Error,
    > {
        let cpal_host = cpal::default_host();
        println!("Cpal Host: {:?}", &cpal_host.id());

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
}

pub struct BackendBuilder<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>
where
    TCaptureSample: CompatibleSample + Send + Sync,
    TPlaybackSample: CompatibleSample + Send + Sync,

    TCaptureDataWriter: AsyncWriteItems<TCaptureSample> + Unpin + Send,
    TPlaybackDataReader: AsyncReadItems<TPlaybackSample> + Unpin + Send,
{
    pub continuation: FormatNegotiationContinuation<TCaptureSample, TPlaybackSample>,

    pub shared_capture_data_writer: Synced<TCaptureDataWriter>,
    pub shared_playback_data_reader: Synced<TPlaybackDataReader>,
}

impl<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader> audio::BackendBuilder
    for BackendBuilder<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>
where
    TCaptureSample: CompatibleSample + Send + Sync,
    TPlaybackSample: CompatibleSample + Send + Sync,

    TCaptureDataWriter: AsyncWriteItems<TCaptureSample> + Unpin + Send,
    TPlaybackDataReader: AsyncReadItems<TPlaybackSample> + Unpin + Send,
{
    type Backend =
        Backend<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>;

    fn build(self) -> Result<Self::Backend, crate::Error> {
        let cpal_capture_format = format::to_cpal_format(self.continuation.capture_format);
        let cpal_playback_format = format::to_cpal_format(self.continuation.playback_format);

        print_config(
            "Playback",
            &self.continuation.cpal_output_device.name()?,
            &cpal_playback_format,
        );
        print_config(
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

            capture_data_writer: self.shared_capture_data_writer,
            playback_data_reader: self.shared_playback_data_reader,

            capture_stream_id,
            playback_stream_id,

            cpal_event_loop,
        };
        Ok(backend)
    }
}

fn print_config(name: &'static str, device_name: &str, format: &cpal::Format) {
    println!("{} device: {}", name, device_name);
    println!("{} format: {:?}", name, format);
    println!(
        "{} endianness: {}",
        name,
        if cfg!(target_endian = "little") {
            "little"
        } else {
            "big"
        }
    );
    // Always interleaved.
    println!("{} operation mode: interleaved", name);
}
