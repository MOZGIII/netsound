pub fn default() -> Result<(cpal::Device, cpal::Format), super::errors::Error> {
    let device = cpal::default_output_device().ok_or(super::errors::Error::DefaultDeviceError)?;
    let format = device.default_output_format()?;
    println!("Default output device: {}", device.name());
    println!("Default output format: {:?}", format);
    return Ok((device, format));
}

pub fn build_default_stream(
    event_loop: &cpal::EventLoop,
) -> Result<cpal::StreamId, super::errors::Error> {
    let (device, format) = default()?;
    let stream_id = event_loop.build_output_stream(&device, &format)?;
    Ok(stream_id)
}

pub fn extend(from: &mut std::collections::VecDeque<f32>, to: &mut cpal::UnknownTypeOutputBuffer) {
    match to {
        cpal::UnknownTypeOutputBuffer::U16(buffer) => {
            for sample_slot in buffer.iter_mut() {
                let sample = from.pop_front();
                let sample = match sample {
                    None => 0u16,
                    Some(sample) => cpal::Sample::to_u16(&sample),
                };
                *sample_slot = sample;
            }
        }
        cpal::UnknownTypeOutputBuffer::I16(buffer) => {
            for sample_slot in buffer.iter_mut() {
                let sample = from.pop_front();
                let sample = match sample {
                    None => 0i16,
                    Some(sample) => cpal::Sample::to_i16(&sample),
                };
                *sample_slot = sample;
            }
        }
        cpal::UnknownTypeOutputBuffer::F32(buffer) => {
            for sample_slot in buffer.iter_mut() {
                let sample = from.pop_front();
                let sample = match sample {
                    None => 0.0,
                    Some(sample) => cpal::Sample::to_f32(&sample),
                };
                *sample_slot = sample;
            }
        }
    }
}
