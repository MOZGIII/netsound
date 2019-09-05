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

impl<'a, TCaptureDataWriter, TPlaybackDataReader>
    audio::BackendBuilderFor<Backend<f32, f32, TCaptureDataWriter, TPlaybackDataReader>>
    for audio::BackendBuilder<'a, f32, f32, TCaptureDataWriter, TPlaybackDataReader>
where
    TCaptureDataWriter: WriteItems<f32> + Send,
    TPlaybackDataReader: ReadItems<f32> + Send,
{
    fn build(self) -> Result<Backend<f32, f32, TCaptureDataWriter, TPlaybackDataReader>, Error> {
        let pa_record = util::build_psimple(Direction::Record);
        let pa_playback = util::build_psimple(Direction::Playback);
        Ok(Backend {
            capture_sample: PhantomData,
            playback_sample: PhantomData,

            capture_data_writer: self.capture_data_writer,
            playback_data_reader: self.playback_data_reader,

            pa_record,
            pa_playback,
        })
    }
}

impl<TCaptureDataWriter, TPlaybackDataReader> audio::Backend
    for Backend<f32, f32, TCaptureDataWriter, TPlaybackDataReader>
where
    TCaptureDataWriter: WriteItems<f32> + Send,
    TPlaybackDataReader: ReadItems<f32> + Send,
{
    type CaptureSample = f32;
    type PlaybackSample = f32;

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

    fn capture_format(&self) -> Format<f32> {
        // TODO: implement properly.
        Format::new(2, 48000)
    }

    fn playback_format(&self) -> Format<f32> {
        // TODO: implement properly.
        Format::new(2, 48000)
    }
}
