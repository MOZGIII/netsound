use crate::io::{ReadSamples, WriteSamples};
use byteorder::ByteOrder;
use std::io::Result;

#[cfg(test)]
use crate::buf::VecDequeSampleBuffer;

pub fn encode<E: ByteOrder, T: ReadSamples<f32>>(
    input: &mut T,
    output: &mut [u8],
) -> Result<usize> {
    // TODO: implement more efficiently.

    // Get the amount of samples to read. Must be round, otherwise we can't
    // fit all the samples we read to the output.
    let samples_to_read = output.len() / 4;
    assert_eq!(samples_to_read * 4, output.len());
    dbg!(samples_to_read);

    let mut samples = Vec::with_capacity(samples_to_read);
    samples.resize(samples_to_read, 0f32);

    let samples_read = input.read_samples(&mut samples)?;
    dbg!(samples_read);

    for (mut chunk, &sample) in output.chunks_exact_mut(4).zip(&samples[..samples_read]) {
        E::write_f32(&mut chunk, sample);
    }

    Ok(samples_read * 4)
}

#[test]
fn test_encode() {
    let mut output = [0u8; 32];
    let mut input: VecDequeSampleBuffer<f32> = (0..1024).map(|sample| sample as f32).collect();
    let f32_size_in_bytes = 32 / 8;

    use byteorder::BigEndian;
    let filled = encode::<BigEndian, VecDequeSampleBuffer<f32>>(&mut input, &mut output).unwrap();

    // The whole buffer should be filled in.
    assert_eq!(filled, 32);

    // VecDeque's length should've decreased appropriately.
    assert_eq!(input.len(), 1024 - (filled / f32_size_in_bytes));
}

#[test]
fn test_encode_values() {
    let mut output = [0u8; 32];
    let mut input = VecDequeSampleBuffer::from(vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);

    let f32_size_in_bytes = std::mem::size_of::<f32>();
    let input_len_before_op = input.len();

    use byteorder::BigEndian;
    let filled = encode::<BigEndian, VecDequeSampleBuffer<f32>>(&mut input, &mut output).unwrap();
    assert_eq!(filled, input_len_before_op * f32_size_in_bytes);
    assert_eq!(input.len(), 0);

    assert_eq!(
        &output[..filled],
        &[
            /* 0.0 -------------------------------------------------------- */
            0x00, 0x00, 0x00, 0x00,
            /* 1.0 -------------------------------------------------------- */
            0x3F, 0x80, 0x00, 0x00,
            /* 2.0 -------------------------------------------------------- */
            0x40, 0x00, 0x00, 0x00,
            /* 3.0 -------------------------------------------------------- */
            0x40, 0x40, 0x00, 0x00,
            /* 4.0 -------------------------------------------------------- */
            0x40, 0x80, 0x00, 0x00,
            /* 5.0 -------------------------------------------------------- */
            0x40, 0xA0, 0x00, 0x00,
            /* 6.0 -------------------------------------------------------- */
            0x40, 0xC0, 0x00, 0x00,
            /* 7.0 -------------------------------------------------------- */
            0x40, 0xE0, 0x00, 0x00,
        ]
    );
}

pub fn decode<E: ByteOrder, T: WriteSamples<f32>>(input: &[u8], output: &mut T) -> Result<usize> {
    // TODO: implement more efficiently.

    // Get the amount of samples to write. Must be round, otherwise we can't
    // prepare the samples buffer for all bytes we have at the input.
    let samples_to_write = input.len() / 4;
    assert_eq!(samples_to_write * 4, input.len());

    let mut samples = Vec::with_capacity(samples_to_write);
    samples.resize(samples_to_write, 0f32);

    for (chunk, sample_slot) in input.chunks(4).zip(&mut samples) {
        *sample_slot = E::read_f32(&chunk);
    }

    output.write_samples(&samples)
}

#[test]
fn test_decode() {
    let mut output: VecDequeSampleBuffer<f32> = (0..1024).map(|sample| sample as f32).collect();
    let input = [0u8; 32];
    let f32_size_in_bytes = 32 / 8;

    use byteorder::BigEndian;
    decode::<BigEndian, VecDequeSampleBuffer<f32>>(&input, &mut output).unwrap();
    assert_eq!(output.len(), 1024 + (input.len() / f32_size_in_bytes));
}

#[test]
fn test_e2e() {
    let sample: Vec<f32> = (0..1024).map(|sample| sample as f32).collect();

    let mut vecdeque: VecDequeSampleBuffer<f32> = From::from(sample.clone());
    let mut send_recv_buf = [0u8; 32];
    let f32_size_in_bytes = 32 / 8;
    let samples_per_op = send_recv_buf.len() / f32_size_in_bytes;

    let full_rotation = vecdeque.len() / samples_per_op;

    for _ in 0..full_rotation {
        // dbg!(&vecdeque);

        use byteorder::BigEndian;
        let filled =
            encode::<BigEndian, VecDequeSampleBuffer<f32>>(&mut vecdeque, &mut send_recv_buf)
                .unwrap();
        assert_eq!(filled, send_recv_buf.len());
        assert_eq!(vecdeque.len(), sample.len() - samples_per_op);

        // dbg!(&vecdeque);

        decode::<BigEndian, VecDequeSampleBuffer<f32>>(&send_recv_buf, &mut vecdeque).unwrap();
        assert_eq!(vecdeque.len(), sample.len());

        // dbg!(&vecdeque);
    }

    let result: Vec<f32> = vecdeque.into();
    assert!(sample == result);
}
