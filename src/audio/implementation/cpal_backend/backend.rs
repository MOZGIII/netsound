use super::*;
use crate::audio;
use crate::io::{AsyncReadItems, AsyncWriteItems};
use crate::log::*;
use crate::sample::Sample;
use cpal::traits::*;
use futures::executor::block_on;
use std::marker::PhantomData;

pub struct Backend<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>
where
    TCaptureSample: Sample,
    TPlaybackSample: Sample,

    TCaptureDataWriter: AsyncWriteItems<TCaptureSample>,
    TPlaybackDataReader: AsyncReadItems<TPlaybackSample>,
{
    pub(super) capture_sample: PhantomData<TCaptureSample>,
    pub(super) playback_sample: PhantomData<TPlaybackSample>,

    pub(super) capture_data_writer: TCaptureDataWriter,
    pub(super) playback_data_reader: TPlaybackDataReader,

    #[allow(dead_code)]
    pub(super) capture_stream_id: cpal::StreamId,
    #[allow(dead_code)]
    pub(super) playback_stream_id: cpal::StreamId,

    pub(super) cpal_event_loop: cpal::EventLoop,
}

impl<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader> audio::Backend
    for Backend<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>
where
    TCaptureSample: CompatibleSample + Send + Sync,
    TPlaybackSample: CompatibleSample + Send + Sync,

    TCaptureDataWriter: AsyncWriteItems<TCaptureSample> + Unpin + Send + Sync,
    TPlaybackDataReader: AsyncReadItems<TPlaybackSample> + Unpin + Send + Sync,
{
    fn run(&mut self) {
        let capture_data_writer = &mut self.capture_data_writer;
        let playback_data_reader = &mut self.playback_data_reader;

        self.cpal_event_loop.run(move |stream_id, stream_result| {
            let stream_data = match stream_result {
                Ok(data) => data,
                Err(err) => {
                    crit!("an error occurred on stream {:?}: {}", stream_id, err);
                    return;
                }
            };
            match stream_data {
                cpal::StreamData::Input {
                    buffer: mut input_buf,
                } => {
                    block_on(async {
                        trace!("cpal: before capture");
                        io::capture(&mut input_buf, capture_data_writer).await;
                        trace!("cpal: after capture");
                    });
                }
                cpal::StreamData::Output {
                    buffer: mut output_buf,
                } => {
                    block_on(async {
                        trace!("cpal: before play");
                        io::play(playback_data_reader, &mut output_buf).await;
                        trace!("cpal: after play");
                    });
                }
            };
        });
    }
}
