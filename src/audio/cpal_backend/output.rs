pub fn extend(
    from: &mut std::collections::VecDeque<f32>,
    to: &mut cpal::UnknownTypeOutputBuffer<'_>,
) {
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
