pub fn extend(from: cpal::UnknownTypeInputBuffer<'_>, to: &mut std::collections::VecDeque<f32>) {
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
