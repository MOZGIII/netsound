extern crate cpal;

use parking_lot::Mutex;
use std::collections::VecDeque;
use std::error::Error;
use std::sync::Arc;

mod cpal_backend;

#[cfg(feature = "pulse_simple_backend")]
pub mod pulse_simple_backend;

pub trait Backend {
    fn run(self);
}

pub struct BackendBuilder {
    pub capture_buf: Arc<Mutex<VecDeque<f32>>>,
    pub playback_buf: Arc<Mutex<VecDeque<f32>>>,
}

pub trait BackendBuilderFor<T: Backend> {
    fn build(self) -> Result<T, Box<Error>>;
}

pub struct Cpal {
    service: cpal_backend::AudioService,
    cpal_eventloop: cpal::EventLoop,
}

impl BackendBuilderFor<Cpal> for BackendBuilder {
    fn build(self) -> Result<Cpal, Box<Error>> {
        let evl = cpal_backend::prepare_cpal_loop()?;
        Ok(Cpal {
            cpal_eventloop: evl,
            service: cpal_backend::AudioService {
                input_buf: self.capture_buf,
                output_buf: self.playback_buf,
            },
        })
    }
}

impl Backend for Cpal {
    fn run(self) {
        self.service.run_cpal_loop(self.cpal_eventloop);
    }
}
