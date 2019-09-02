use super::*;
use crate::audio;
use crate::io::{ReadSamples, WriteSamples};
use crate::sync::Synced;
use cpal::traits::*;
use sample::Sample;
use std::marker::PhantomData;

pub struct Backend<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>
where
    TCaptureSample: Sample + Send,
    TPlaybackSample: Sample + Send,
    TCaptureDataWriter: WriteSamples<TCaptureSample>,
    TPlaybackDataReader: ReadSamples<TPlaybackSample>,
{
    pub(super) capture_sample: PhantomData<TCaptureSample>,
    pub(super) playback_sample: PhantomData<TPlaybackSample>,

    pub(super) capture_data_writer: Synced<TCaptureDataWriter>,
    pub(super) playback_data_reader: Synced<TPlaybackDataReader>,

    pub(super) capture_format: cpal::Format,
    pub(super) playback_format: cpal::Format,

    #[allow(dead_code)]
    pub(super) capture_stream_id: cpal::StreamId,
    #[allow(dead_code)]
    pub(super) playback_stream_id: cpal::StreamId,

    pub(super) cpal_eventloop: cpal::EventLoop,
}

impl<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader> audio::Backend
    for Backend<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>
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
    fn run(&mut self) {
        let input_converter = conv::ExactCpalInputConverter::new();
        let output_converter = conv::ExactCpalOutputConverter::new();
        let capture_data_writer = &self.capture_data_writer;
        let playback_data_reader = &self.playback_data_reader;

        self.cpal_eventloop.run(move |stream_id, stream_result| {
            let stream_data = match stream_result {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("an error occurred on stream {:?}: {}", stream_id, err);
                    return;
                }
            };
            match stream_data {
                cpal::StreamData::Input {
                    buffer: mut input_buf,
                } => {
                    let mut capture_data_writer_guard = capture_data_writer.lock();
                    io::capture(
                        &input_converter,
                        &mut input_buf,
                        &mut *capture_data_writer_guard,
                    )
                }
                cpal::StreamData::Output {
                    buffer: mut output_buf,
                } => {
                    let mut playback_data_reader_guard = playback_data_reader.lock();
                    io::play(
                        &output_converter,
                        &mut *playback_data_reader_guard,
                        &mut output_buf,
                    )
                }
            };
        });
    }

    fn capture_format(&self) -> audio::Format {
        audio::Format::from(&self.capture_format)
    }

    fn playback_format(&self) -> audio::Format {
        audio::Format::from(&self.playback_format)
    }
}

impl From<&cpal::Format> for audio::Format {
    fn from(f: &cpal::Format) -> Self {
        Self {
            channels: f.channels,
            sample_rate: f.sample_rate.0,
        }
    }
}

impl Into<cpal::Format> for &audio::Format {
    fn into(self) -> cpal::Format {
        cpal::Format {
            channels: self.channels,
            sample_rate: cpal::SampleRate(self.sample_rate),
            data_type: cpal::SampleFormat::F32,
        }
    }
}
