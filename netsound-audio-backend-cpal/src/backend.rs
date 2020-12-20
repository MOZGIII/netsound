use super::{io, CompatibleSample};
use cpal::traits::EventLoopTrait;
use netsound_core::audio_backend;
use netsound_core::io::{AsyncReadItems, AsyncWriteItems};
use netsound_core::log::no_scopes::{crit, trace, Logger};
use netsound_core::sample::Sample;
use std::marker::PhantomData;

#[derive(Derivative)]
#[derivative(Debug)]
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

    pub(super) capture_stream_id: cpal::StreamId,
    pub(super) playback_stream_id: cpal::StreamId,

    #[derivative(Debug = "ignore")]
    pub(super) cpal_event_loop: cpal::EventLoop,

    pub(super) logger: Logger,
}

impl<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>
    audio_backend::Backend
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
        let logger = &mut self.logger;

        self.cpal_event_loop.run(move |stream_id, stream_result| {
            trace!(logger, "cpal: at event loop");
            let stream_data = match stream_result {
                Ok(data) => data,
                Err(err) => {
                    crit!(
                        logger,
                        "an error occurred on stream {:?}: {}",
                        stream_id,
                        err
                    );
                    return;
                }
            };
            match stream_data {
                cpal::StreamData::Input {
                    buffer: mut input_buf,
                } => {
                    trace!(logger, "cpal: before capture");
                    io::capture(&mut input_buf, capture_data_writer);
                    trace!(logger, "cpal: after capture");
                }
                cpal::StreamData::Output {
                    buffer: mut output_buf,
                } => {
                    trace!(logger, "cpal: before play");
                    io::play(playback_data_reader, &mut output_buf);
                    trace!(logger, "cpal: after play");
                }
            };
        });
    }
}
