use byteorder::ByteOrder;
use std::collections::VecDeque;

pub fn encode<E: ByteOrder>(input: &mut VecDeque<f32>, output: &mut [u8]) -> usize {
    let mut filled = 0;
    for mut chunk in output.chunks_exact_mut(4) {
        match input.pop_front() {
            None => break,
            Some(sample) => {
                E::write_f32(&mut chunk, sample);
                filled += 4;
            }
        }
    }
    filled
}

#[test]
fn test_encode() {
    let mut output = [0u8; 32];
    let mut input: VecDeque<f32> = (0..1024).map(|sample| sample as f32).collect();
    let f32_size_in_bytes = 32 / 8;

    use byteorder::BigEndian;
    let filled = encode::<BigEndian>(&mut input, &mut output);

    // The whole buffer should be filled in.
    assert_eq!(filled, 32);

    // VecDeque's length should've decreased appropriately.
    assert_eq!(input.len(), 1024 - (filled / f32_size_in_bytes));
}

#[test]
fn test_encode_values() {
    let mut output = [0u8; 32];
    let mut input = VecDeque::from(vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);

    let f32_size_in_bytes = std::mem::size_of::<f32>();
    let input_len_before_op = input.len();

    use byteorder::BigEndian;
    let filled = encode::<BigEndian>(&mut input, &mut output);
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

pub fn decode<E: ByteOrder>(input: &[u8], output: &mut VecDeque<f32>) -> usize {
    let mut filled = 0;
    for chunk in input.chunks(4) {
        let sample = E::read_f32(&chunk);
        output.push_back(sample);
        filled += 1;
    }
    filled
}

#[test]
fn test_decode() {
    let mut output: VecDeque<f32> = (0..1024).map(|sample| sample as f32).collect();
    let input = [0u8; 32];
    let f32_size_in_bytes = 32 / 8;

    use byteorder::BigEndian;
    decode::<BigEndian>(&input, &mut output);
    assert_eq!(output.len(), 1024 + (input.len() / f32_size_in_bytes));
}

#[test]
fn test_e2e() {
    let sample: Vec<f32> = (0..1024).map(|sample| sample as f32).collect();

    let mut vecdeque: VecDeque<f32> = From::from(sample.clone());
    let mut send_recv_buf = [0u8; 32];
    let f32_size_in_bytes = 32 / 8;
    let samples_per_op = send_recv_buf.len() / f32_size_in_bytes;

    let full_rotation = vecdeque.len() / samples_per_op;

    for _ in 0..full_rotation {
        // dbg!(&vecdeque);

        use byteorder::BigEndian;
        let filled = encode::<BigEndian>(&mut vecdeque, &mut send_recv_buf);
        assert_eq!(filled, send_recv_buf.len());
        assert_eq!(vecdeque.len(), sample.len() - samples_per_op);

        // dbg!(&vecdeque);

        decode::<BigEndian>(&send_recv_buf, &mut vecdeque);
        assert_eq!(vecdeque.len(), sample.len());

        // dbg!(&vecdeque);
    }

    let result: Vec<f32> = vecdeque.into();
    assert!(sample == result);
}
