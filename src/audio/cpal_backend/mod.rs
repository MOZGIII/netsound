use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;

pub mod errors;
pub mod input;
pub mod output;

fn prepare_cpal_loop() -> Result<cpal::EventLoop, errors::Error> {
    let event_loop = cpal::EventLoop::new();

    let input_stream_id = input::build_default_stream(&event_loop)?;
    event_loop.play_stream(input_stream_id);

    let output_stream_id = output::build_default_stream(&event_loop)?;
    event_loop.play_stream(output_stream_id);

    Ok(event_loop)
}

pub struct Backend {
    input_buf: Arc<Mutex<VecDeque<f32>>>,
    output_buf: Arc<Mutex<VecDeque<f32>>>,
    cpal_eventloop: cpal::EventLoop,
}

impl super::BackendBuilderFor<Backend> for super::BackendBuilder {
    fn build(self) -> Result<Backend, Box<std::error::Error>> {
        let evl = prepare_cpal_loop()?;
        Ok(Backend {
            input_buf: self.capture_buf,
            output_buf: self.playback_buf,
            cpal_eventloop: evl,
        })
    }
}

impl super::Backend for Backend {
    fn run(self) {
        let shared_input_buf = self.input_buf;
        let shared_output_buf = self.output_buf;
        self.cpal_eventloop.run(move |_, data| {
            match data {
                cpal::StreamData::Input { buffer: input_buf } => {
                    let mut self_input_buf = shared_input_buf.lock();
                    self_input_buf.reserve(input_buf.len());
                    input::extend(input_buf, &mut self_input_buf);
                }
                cpal::StreamData::Output {
                    buffer: mut output_buf,
                } => {
                    let mut self_output_buf = shared_output_buf.lock();
                    output::extend(&mut self_output_buf, &mut output_buf);
                }
            };
        });
    }
}
