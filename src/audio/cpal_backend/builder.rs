use super::*;
use crate::audio;
use crate::io::{ReadSamples, WriteSamples};
use sample::Sample;
use std::marker::PhantomData;

use cpal::traits::*;

fn build<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>(
    builder: audio::BackendBuilder<'_, TCaptureDataWriter, TPlaybackDataReader>,
) -> Result<
    Backend<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>,
    errors::Error,
>
where
    TCaptureSample: Sample + Send,
    TPlaybackSample: Sample + Send,
    TCaptureDataWriter: WriteSamples<TCaptureSample>,
    TPlaybackDataReader: ReadSamples<TPlaybackSample>,
{
    let host = cpal::default_host();
    println!("Cpal Host: {:?}", &host.id());

    let event_loop = host.event_loop();

    let input_device = default::input_device(&host)?;
    let output_device = default::output_device(&host)?;

    let capture_format = format::choose(
        input_device.supported_input_formats()?,
        builder.request_capture_formats,
    )?;
    let playback_format = format::choose(
        output_device.supported_output_formats()?,
        builder.request_playback_formats,
    )?;

    print_config("Playback", &output_device.name()?, &playback_format);
    print_config("Capture", &input_device.name()?, &capture_format);

    let playback_stream_id = event_loop.build_output_stream(&output_device, &playback_format)?;
    let capture_stream_id = event_loop.build_input_stream(&input_device, &capture_format)?;

    event_loop.play_stream(playback_stream_id.clone())?;
    event_loop.play_stream(capture_stream_id.clone())?;

    Ok(Backend {
        capture_sample: PhantomData,
        playback_sample: PhantomData,

        capture_data_writer: builder.capture_data_writer,
        playback_data_reader: builder.playback_data_reader,

        capture_format,
        playback_format,

        capture_stream_id,
        playback_stream_id,

        cpal_eventloop: event_loop,
    })
}

impl<'a, TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>
    audio::BackendBuilderFor<
        Backend<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>,
    > for audio::BackendBuilder<'a, TCaptureDataWriter, TPlaybackDataReader>
where
    TCaptureSample: Sample + Send + Sync,
    TPlaybackSample: Sample + Send + Sync,

    TCaptureDataWriter: WriteSamples<TCaptureSample> + Send,
    TPlaybackDataReader: ReadSamples<TPlaybackSample> + Send,

    conv::ExactCpalInputConverter<TCaptureSample>:
        conv::CpalInputConverter<Sample = TCaptureSample>,
    conv::ExactCpalOutputConverter<TPlaybackSample>:
        conv::CpalOutputConverter<Sample = TPlaybackSample>,
{
    fn build(
        self,
    ) -> Result<
        Backend<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>,
        crate::Error,
    > {
        Ok(build(self)?)
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
