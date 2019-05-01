use byteorder::{ByteOrder, NativeEndian};
use crossbeam_utils;
use libpulse_binding as pulse;
use libpulse_simple_binding as psimple;

use self::crossbeam_utils::thread;
use self::psimple::Simple;
use self::pulse::error::PAErr;
use self::pulse::stream::Direction;

use crate::samples::SharedSamples;
use std::error::Error;

mod util;

pub struct Backend {
    playback_buf: SharedSamples,
    #[allow(dead_code)]
    record_buf: SharedSamples,
    pa_playback: Simple,
    #[allow(dead_code)]
    pa_record: Simple,
}

impl<'a> super::BackendBuilderFor<Backend> for super::BackendBuilder<'a> {
    fn build(self) -> Result<Backend, Box<dyn Error>> {
        let pa_playback = util::build_psimple(Direction::Playback);
        let pa_record = util::build_psimple(Direction::Record);
        Ok(Backend {
            playback_buf: self.playback_buf,
            record_buf: self.capture_buf,
            pa_playback: pa_playback,
            pa_record: pa_record,
        })
    }
}

impl super::Backend for Backend {
    fn run(&mut self) {
        let playback_buf = &mut self.playback_buf;
        let pa_playback = &mut self.pa_playback;
        let record_buf = &mut self.record_buf;
        let pa_record = &mut self.pa_record;

        thread::scope(|s| {
            let mut buff = [0u8; 128];
            let playback_handle = s.spawn(move |_| {
                loop {
                    // Play what's in playback buffer.
                    let mut playback_buf = playback_buf.lock();

                    let mut filled = 0;
                    for mut chunk in buff.chunks_exact_mut(4) {
                        match playback_buf.pop_front() {
                            None => break,
                            Some(sample) => {
                                NativeEndian::write_f32(&mut chunk, sample);
                                filled += 4;
                            }
                        }
                    }

                    let write_buff = &buff[..filled];

                    if write_buff.len() > 0 {
                        if let Err(PAErr(err)) = pa_playback.write(&buff[..filled]) {
                            dbg!(err);
                            break;
                        }
                    }
                }
            });
            let record_handle = s.spawn(move |_| {
                let mut buff = [0u8; 128];
                loop {
                    // Record to record buffer.
                    let size: usize = 128;
                    let read_buff = &mut buff[..size];
                    if let Err(PAErr(err)) = pa_record.read(read_buff) {
                        dbg!(err);
                        break;
                    }

                    let mut record_buf = record_buf.lock();
                    for mut chunk in read_buff.chunks_exact_mut(4) {
                        let sample = NativeEndian::read_f32(&mut chunk);
                        record_buf.push_back(sample);
                    }
                }
            });

            playback_handle.join().unwrap();
            record_handle.join().unwrap();
        })
        .unwrap()
    }

    fn capture_format(&self) -> super::Format {
        // TODO: implement properly.
        super::Format {
            channels: 2,
            sample_rate: 48000,
        }
    }

    fn playback_format(&self) -> super::Format {
        // TODO: implement properly.
        super::Format {
            channels: 2,
            sample_rate: 48000,
        }
    }
}
