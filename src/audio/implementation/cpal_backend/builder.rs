use super::{choose_format::choose_format, *};
use crate::audio;
use crate::format::Format;
use crate::io::{ReadItems, WriteItems};
use crate::sync::Synced;
use sample::Sample;
use std::marker::PhantomData;

use cpal::traits::*;

impl<
        'a,
        TCaptureSample,
        TPlaybackSample,
        TCaptureData,
        TPlaybackData,
        TSharedCaptureDataBuilder,
        TSharedPlaybackDataBuilder,
    >
    audio::Build<
        Backend<TCaptureSample, TPlaybackSample, TCaptureData, TPlaybackData>,
        TCaptureSample,
        TPlaybackSample,
        TCaptureData,
        TPlaybackData,
        TSharedCaptureDataBuilder,
        TSharedPlaybackDataBuilder,
    >
    for audio::Builder<
        'a,
        TCaptureSample,
        TPlaybackSample,
        TCaptureData,
        TPlaybackData,
        TSharedCaptureDataBuilder,
        TSharedPlaybackDataBuilder,
    >
where
    TCaptureSample: Sample + Send + Sync,
    TPlaybackSample: Sample + Send + Sync,

    TCaptureData: WriteItems<TCaptureSample> + Send,
    TPlaybackData: ReadItems<TPlaybackSample> + Send,

    TSharedCaptureDataBuilder:
        FnOnce(Format<TCaptureSample>) -> Result<Synced<TCaptureData>, crate::Error>,
    TSharedPlaybackDataBuilder:
        FnOnce(Format<TPlaybackSample>) -> Result<Synced<TPlaybackData>, crate::Error>,

    conv::ExactCpalInputConverter<TCaptureSample>:
        conv::CpalInputConverter<Sample = TCaptureSample>,
    conv::ExactCpalOutputConverter<TPlaybackSample>:
        conv::CpalOutputConverter<Sample = TPlaybackSample>,

    format::deduce::ExactCpalSampleFormatDeducer<TCaptureSample>:
        format::deduce::CpalSampleFormatDeducer<Sample = TCaptureSample>,
    format::deduce::ExactCpalSampleFormatDeducer<TPlaybackSample>:
        format::deduce::CpalSampleFormatDeducer<Sample = TPlaybackSample>,
    format::assert::ExactCpalSampleFormatAsserter<TCaptureSample>:
        format::assert::CpalSampleFormatAsserter<Sample = TCaptureSample>,
    format::assert::ExactCpalSampleFormatAsserter<TPlaybackSample>:
        format::assert::CpalSampleFormatAsserter<Sample = TPlaybackSample>,
{
    fn build(
        self,
    ) -> Result<
        audio::BuiltState<
            Backend<TCaptureSample, TPlaybackSample, TCaptureData, TPlaybackData>,
            TCaptureSample,
            TPlaybackSample,
            Synced<TCaptureData>,
            Synced<TPlaybackData>,
        >,
        crate::Error,
    > {
        let host = cpal::default_host();
        println!("Cpal Host: {:?}", &host.id());

        let event_loop = host.event_loop();

        let input_device = default::input_device(&host)?;
        let output_device = default::output_device(&host)?;

        let capture_format = choose_format(
            input_device.supported_input_formats()?,
            self.request_capture_formats,
        )?;
        let playback_format = choose_format(
            output_device.supported_output_formats()?,
            self.request_playback_formats,
        )?;

        let cpal_capture_format = format::interop::to_cpal_format(capture_format);
        let cpal_playback_format = format::interop::to_cpal_format(playback_format);

        let shared_capture_data_builder = self.shared_capture_data_builder;
        let shared_playback_data_builder = self.shared_playback_data_builder;

        let shared_capture_data = shared_capture_data_builder(capture_format)?;
        let shared_playback_data = shared_playback_data_builder(playback_format)?;

        print_config("Playback", &output_device.name()?, &cpal_capture_format);
        print_config("Capture", &input_device.name()?, &cpal_playback_format);

        let playback_stream_id =
            event_loop.build_output_stream(&output_device, &cpal_capture_format)?;
        let capture_stream_id =
            event_loop.build_input_stream(&input_device, &cpal_playback_format)?;

        event_loop.play_stream(playback_stream_id.clone())?;
        event_loop.play_stream(capture_stream_id.clone())?;

        let backend = Backend {
            capture_sample: PhantomData,
            playback_sample: PhantomData,

            capture_data_writer: shared_capture_data.clone(),
            playback_data_reader: shared_playback_data.clone(),

            capture_stream_id,
            playback_stream_id,

            cpal_eventloop: event_loop,
        };

        let built_state = audio::BuiltState {
            backend,
            capture_format,
            playback_format,
            shared_capture_data,
            shared_playback_data,
        };

        Ok(built_state)
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
