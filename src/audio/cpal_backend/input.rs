pub fn default() -> Result<(cpal::Device, cpal::Format), super::errors::Error> {
    let device = cpal::default_input_device().ok_or(super::errors::Error::DefaultDeviceError)?;
    let format = device.default_input_format()?;
    println!("Default input device: {}", device.name());
    println!("Default input format: {:?}", format);
    return Ok((device, format));
}

pub fn build_default_stream(
    event_loop: &cpal::EventLoop,
) -> Result<cpal::StreamId, super::errors::Error> {
    let (device, format) = default()?;
    let stream_id = event_loop.build_input_stream(&device, &format)?;
    Ok(stream_id)
}

pub fn extend(from: cpal::UnknownTypeInputBuffer, to: &mut std::collections::VecDeque<f32>) {
    match from {
        cpal::UnknownTypeInputBuffer::U16(buffer) => {
            for sample in buffer.iter() {
                let sample = cpal::Sample::to_f32(sample);
                to.push_back(sample);
            }
        }
        cpal::UnknownTypeInputBuffer::I16(buffer) => {
            for sample in buffer.iter() {
                let sample = cpal::Sample::to_f32(sample);
                to.push_back(sample);
            }
        }
        cpal::UnknownTypeInputBuffer::F32(buffer) => {
            for sample in buffer.iter() {
                let sample = cpal::Sample::to_f32(sample);
                to.push_back(sample);
            }
        }
    }
}
