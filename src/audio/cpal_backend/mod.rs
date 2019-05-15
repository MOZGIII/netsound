use crate::samples::SharedSamples;

pub mod default;
pub mod errors;
pub mod format;
pub mod input;
pub mod output;

pub struct Backend {
    input_buf: SharedSamples,
    output_buf: SharedSamples,

    input_format: cpal::Format,
    output_format: cpal::Format,

    #[allow(dead_code)]
    input_stream_id: cpal::StreamId,
    #[allow(dead_code)]
    output_stream_id: cpal::StreamId,

    cpal_eventloop: cpal::EventLoop,
}

impl<'a> super::BackendBuilderFor<Backend> for super::BackendBuilder<'a> {
    fn build(self) -> Result<Backend, Box<dyn std::error::Error>> {
        Ok(build(self)?)
    }
}

fn build(builder: super::BackendBuilder<'_>) -> Result<Backend, errors::Error> {
    let event_loop = cpal::EventLoop::new();

    let output_device = default::output_device()?;
    let input_device = default::input_device()?;

    let output_format = format::choose(
        &mut output_device.supported_output_formats()?,
        builder.request_playback_formats,
    )?;
    let input_format = format::choose(
        &mut input_device.supported_input_formats()?,
        builder.request_capture_formats,
    )?;

    print_config("Input", &input_device, &input_format);
    print_config("Output", &output_device, &output_format);

    let output_stream_id = event_loop.build_output_stream(&output_device, &output_format)?;
    let input_stream_id = event_loop.build_input_stream(&input_device, &input_format)?;

    event_loop.play_stream(output_stream_id.clone());
    event_loop.play_stream(input_stream_id.clone());

    Ok(Backend {
        input_buf: builder.capture_buf,
        output_buf: builder.playback_buf,

        input_format: input_format,
        output_format: output_format,

        input_stream_id: input_stream_id,
        output_stream_id: output_stream_id,

        cpal_eventloop: event_loop,
    })
}

impl super::Backend for Backend {
    fn run(&mut self) {
        let shared_input_buf = &self.input_buf;
        let shared_output_buf = &self.output_buf;
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

    fn capture_format(&self) -> super::Format {
        super::Format::from(&self.input_format)
    }

    fn playback_format(&self) -> super::Format {
        super::Format::from(&self.output_format)
    }
}

impl From<&cpal::Format> for super::Format {
    fn from(f: &cpal::Format) -> Self {
        super::Format {
            channels: f.channels,
            sample_rate: f.sample_rate.0,
        }
    }
}

impl Into<cpal::Format> for &super::Format {
    fn into(self) -> cpal::Format {
        cpal::Format {
            channels: self.channels.into(),
            sample_rate: cpal::SampleRate(self.sample_rate),
            data_type: cpal::SampleFormat::F32,
        }
    }
}

fn print_config(name: &'static str, device: &cpal::Device, format: &cpal::Format) {
    println!("{} device: {}", name, device.name());
    println!("{} format: {:?}", name, format);
    println!(
        "{} endianness: {}",
        name,
        if cfg!(target_endian = "little") {
            "little"
        } else {
            "big"
        }
    );
}
