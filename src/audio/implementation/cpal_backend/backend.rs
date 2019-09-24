use super::*;
use crate::audio;
use crate::io::{AsyncReadItems, AsyncWriteItems};
use crate::sample::Sample;
use crate::sync::Synced;
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

    pub(super) capture_data_writer: Synced<TCaptureDataWriter>,
    pub(super) playback_data_reader: Synced<TPlaybackDataReader>,

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

    TCaptureDataWriter: AsyncWriteItems<TCaptureSample> + Send + Unpin,
    TPlaybackDataReader: AsyncReadItems<TPlaybackSample> + Send + Unpin,
{
    fn run(&mut self) {
        let capture_data_writer = &self.capture_data_writer;
        let playback_data_reader = &self.playback_data_reader;

        self.cpal_event_loop.run(move |stream_id, stream_result| {
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
                    block_on(async {
                        let mut capture_data_writer_guard = capture_data_writer.lock().await;
                        io::capture(&mut input_buf, &mut *capture_data_writer_guard).await;
                    });
                }
                cpal::StreamData::Output {
                    buffer: mut output_buf,
                } => {
                    block_on(async {
                        let mut playback_data_reader_guard = playback_data_reader.lock().await;
                        io::play(&mut *playback_data_reader_guard, &mut output_buf).await;
                    });
                }
            };
        });
    }
}
