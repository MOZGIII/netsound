extern crate libpulse_binding as pulse;
extern crate libpulse_simple_binding as psimple;
use byteorder::{ByteOrder, NativeEndian};

use self::psimple::Simple;
use self::pulse::error::PAErr;
use self::pulse::stream::Direction;

use std::error::Error;
use crate::samples::SharedSamples;

mod util;

pub struct Backend {
    playback_buf: SharedSamples,
    #[allow(dead_code)]
    record_buf: SharedSamples,
    pa_playback: Simple,
    #[allow(dead_code)]
    pa_record: Simple,
}

impl super::BackendBuilderFor<Backend> for super::BackendBuilder {
    fn build(self) -> Result<Backend, Box<Error>> {
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
    fn run(self) {
        let mut buff = [0u8; 102400 * 4];

        loop {
            {
                let mut playback_buf = self.playback_buf.lock();

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
                    if let Err(PAErr(err)) = self.pa_playback.write(&buff[..filled]) {
                        dbg!(err);
                        break;
                    }
                }
            }
        }
    }
}
