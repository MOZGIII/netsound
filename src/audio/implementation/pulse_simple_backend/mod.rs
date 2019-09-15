use crate::audio;
use crate::format::Format;
use crate::io::{ReadItems, WriteItems};
use crate::sync::Synced;
use crossbeam_utils;
use libpulse_binding as pulse;
use libpulse_simple_binding as psimple;
use sample::Sample;
use std::marker::PhantomData;

use self::crossbeam_utils::thread;
use self::psimple::Simple;
use self::pulse::error::PAErr;
use self::pulse::stream::Direction;

use crate::Error;

mod util;

pub struct Backend<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>
where
    TCaptureSample: Sample + Send,
    TPlaybackSample: Sample + Send,
    TCaptureDataWriter: WriteItems<TCaptureSample>,
    TPlaybackDataReader: ReadItems<TPlaybackSample>,
{
    capture_sample: PhantomData<TCaptureSample>,
    playback_sample: PhantomData<TPlaybackSample>,

    capture_data_writer: Synced<TCaptureDataWriter>,
    playback_data_reader: Synced<TPlaybackDataReader>,

    pa_record: Simple,
    pa_playback: Simple,
}

impl<'a, TCaptureData, TPlaybackData, TSharedCaptureDataBuilder, TSharedPlaybackDataBuilder>
    audio::Build<
        Backend<f32, f32, TCaptureData, TPlaybackData>,
        f32,
        f32,
        TCaptureData,
        TPlaybackData,
        TSharedCaptureDataBuilder,
        TSharedPlaybackDataBuilder,
    >
    for audio::Builder<
        'a,
        f32,
        f32,
        TCaptureData,
        TPlaybackData,
        TSharedCaptureDataBuilder,
        TSharedPlaybackDataBuilder,
    >
where
    TCaptureData: WriteItems<f32> + Send,
    TPlaybackData: ReadItems<f32> + Send,
    TSharedCaptureDataBuilder: FnOnce(Format<f32>) -> Result<Synced<TCaptureData>, crate::Error>,
    TSharedPlaybackDataBuilder: FnOnce(Format<f32>) -> Result<Synced<TPlaybackData>, crate::Error>,
{
    fn build(
        self,
    ) -> Result<
        audio::BuiltState<
            Backend<f32, f32, TCaptureData, TPlaybackData>,
            f32,
            f32,
            Synced<TCaptureData>,
            Synced<TPlaybackData>,
        >,
        Error,
    > {
        let pa_record = util::build_psimple(Direction::Record);
        let pa_playback = util::build_psimple(Direction::Playback);

        let capture_format = Format::new(2, 48000);
        let playback_format = Format::new(2, 48000);

        let shared_capture_data_builder = self.shared_capture_data_builder;
        let shared_playback_data_builder = self.shared_playback_data_builder;

        let shared_capture_data = shared_capture_data_builder(capture_format)?;
        let shared_playback_data = shared_playback_data_builder(playback_format)?;

        let backend = Backend {
            capture_sample: PhantomData,
            playback_sample: PhantomData,

            capture_data_writer: shared_capture_data.clone(),
            playback_data_reader: shared_playback_data.clone(),

            pa_record,
            pa_playback,
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

impl<TCaptureDataWriter, TPlaybackDataReader> audio::Backend
    for Backend<f32, f32, TCaptureDataWriter, TPlaybackDataReader>
where
    TCaptureDataWriter: WriteItems<f32> + Send,
    TPlaybackDataReader: ReadItems<f32> + Send,
{
    fn run(&mut self) {
        let capture_data_writer = &mut self.capture_data_writer;
        let playback_data_reader = &mut self.playback_data_reader;
        let pa_record = &mut self.pa_record;
        let pa_playback = &mut self.pa_playback;

        thread::scope(|s| {
            let mut playback_samples = [f32::equilibrium(); 128];
            let mut capture_samples = [f32::equilibrium(); 128];
            let playback_handle = s.spawn(move |_| {
                loop {
                    // Play what's in playback buffer.
                    let mut playback_data_reader = playback_data_reader.lock();

                    let samples_read = (*playback_data_reader)
                        .read_items(&mut playback_samples)
                        .expect("Unable to read playback data");

                    let write_buff = unsafe {
                        std::slice::from_raw_parts(
                            playback_samples.as_ptr() as *const u8,
                            samples_read * 4,
                        )
                    };

                    if !write_buff.is_empty() {
                        if let Err(PAErr(err)) = pa_playback.write(write_buff) {
                            dbg!(err);
                            break;
                        }
                    }
                }
            });
            let record_handle = s.spawn(move |_| {
                loop {
                    // Record to record buffer.
                    {
                        let mut read_buff = unsafe {
                            std::slice::from_raw_parts_mut(
                                capture_samples.as_mut_ptr() as *mut u8,
                                capture_samples.len() * 4,
                            )
                        };

                        if let Err(PAErr(err)) = pa_record.read(&mut read_buff) {
                            dbg!(err);
                            break;
                        }
                    }

                    let mut capture_data_writer = capture_data_writer.lock();
                    let _ = (*capture_data_writer)
                        .write_items(&capture_samples)
                        .expect("Unable to write captured data");
                }
            });

            playback_handle.join().unwrap();
            record_handle.join().unwrap();
        })
        .unwrap()
    }
}
