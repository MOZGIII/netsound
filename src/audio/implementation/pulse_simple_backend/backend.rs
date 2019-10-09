use super::*;
use crate::audio;
use crate::io::{AsyncReadItems, AsyncReadItemsExt, AsyncWriteItems, AsyncWriteItemsExt, WaitMode};
use crossbeam_utils;
use futures::executor::block_on;
use libpulse_binding as pulse;
use libpulse_simple_binding as psimple;
use std::marker::PhantomData;

use self::crossbeam_utils::thread;
use self::psimple::Simple;
use self::pulse::error::PAErr;

pub struct Backend<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>
where
    TCaptureSample: CompatibleSample + Send,
    TPlaybackSample: CompatibleSample + Send,
    TCaptureDataWriter: AsyncWriteItems<TCaptureSample>,
    TPlaybackDataReader: AsyncReadItems<TPlaybackSample>,
{
    pub(super) capture_sample: PhantomData<TCaptureSample>,
    pub(super) playback_sample: PhantomData<TPlaybackSample>,

    pub(super) capture_data_writer: TCaptureDataWriter,
    pub(super) playback_data_reader: TPlaybackDataReader,

    pub(super) pa_record: Simple,
    pub(super) pa_playback: Simple,
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
        let pa_record = &mut self.pa_record;
        let pa_playback = &mut self.pa_playback;

        thread::scope(|s| {
            let mut playback_samples = [TPlaybackSample::equilibrium(); 128];
            let mut capture_samples = [TCaptureSample::equilibrium(); 128];
            let playback_handle = s.spawn(move |_| {
                loop {
                    // Play what's in playback buffer.
                    let samples_read = block_on(
                        playback_data_reader.read_items(&mut playback_samples, WaitMode::NoWait),
                    )
                    .expect("Unable to read playback data");

                    let write_buff = unsafe {
                        std::slice::from_raw_parts(
                            playback_samples.as_ptr() as *const u8,
                            samples_read * std::mem::size_of::<TPlaybackSample>(),
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
                                capture_samples.len() * std::mem::size_of::<TCaptureSample>(),
                            )
                        };

                        if let Err(PAErr(err)) = pa_record.read(&mut read_buff) {
                            dbg!(err);
                            break;
                        }
                    }

                    let _ = block_on(
                        capture_data_writer.write_items(&capture_samples, WaitMode::NoWait),
                    )
                    .expect("Unable to write captured data");
                }
            });

            playback_handle.join().unwrap();
            record_handle.join().unwrap();
        })
        .unwrap()
    }
}
