use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;

pub mod errors;
pub mod input;
pub mod output;

pub fn prepare_cpal_loop() -> Result<cpal::EventLoop, errors::Error> {
    let event_loop = cpal::EventLoop::new();

    let input_stream_id = input::build_default_stream(&event_loop)?;
    event_loop.play_stream(input_stream_id);

    let output_stream_id = output::build_default_stream(&event_loop)?;
    event_loop.play_stream(output_stream_id);

    Ok(event_loop)
}

pub struct AudioService {
    pub input_buf: Arc<Mutex<VecDeque<f32>>>,
    pub output_buf: Arc<Mutex<VecDeque<f32>>>,
}

impl AudioService {
    pub fn run_cpal_loop(&self, event_loop: cpal::EventLoop) {
        event_loop.run(move |_, data| {
            match data {
                cpal::StreamData::Input { buffer: input_buf } => {
                    let mut self_input_buf = self.input_buf.lock();
                    self_input_buf.reserve(input_buf.len());
                    input::extend(input_buf, &mut self_input_buf);
                }
                cpal::StreamData::Output {
                    buffer: mut output_buf,
                } => {
                    let mut self_output_buf = self.output_buf.lock();
                    output::extend(&mut self_output_buf, &mut output_buf);
                }
            };
        });
    }
}
