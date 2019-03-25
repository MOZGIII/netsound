use byteorder::{BigEndian, ByteOrder};
use std::collections::VecDeque;

pub fn vecdec_to_sendbuf(vecdec: &mut VecDeque<f32>, sendbuf: &mut [u8]) -> usize {
    let mut filled = 0;
    for mut chunk in sendbuf.chunks_exact_mut(4) {
        match vecdec.pop_front() {
            None => break,
            Some(sample) => {
                BigEndian::write_f32(&mut chunk, sample);
                filled += 4;
            }
        }
    }
    filled
}

#[test]
fn test_vecdec_to_sendbuf() {
    let mut send_buf = [0u8; 32];
    let mut vecdeque: VecDeque<f32> = (0..1024).map(|sample| sample as f32).collect();
    let f32_size_in_bytes = 32 / 8;

    let filled = vecdec_to_sendbuf(&mut vecdeque, &mut send_buf);

    // The whole buffer should be filled in.
    assert_eq!(filled, 32);

    // VecDeque's length should've decreased appropriately.
    assert_eq!(vecdeque.len(), 1024 - (filled / f32_size_in_bytes));
}

pub fn recvbuf_to_vecdec(recvbuf: &[u8], vecdec: &mut VecDeque<f32>) {
    for chunk in recvbuf.chunks(4) {
        let sample = BigEndian::read_f32(&chunk);
        vecdec.push_back(sample);
    }
}

#[test]
fn test_recvbuf_to_vecdec() {
    let mut vecdeque: VecDeque<f32> = (0..1024).map(|sample| sample as f32).collect();
    let recv_buf = [0u8; 32];
    let f32_size_in_bytes = 32 / 8;

    recvbuf_to_vecdec(&recv_buf, &mut vecdeque);
    assert_eq!(vecdeque.len(), 1024 + (recv_buf.len() / f32_size_in_bytes));
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

        let filled = vecdec_to_sendbuf(&mut vecdeque, &mut send_recv_buf);
        assert_eq!(filled, send_recv_buf.len());
        assert_eq!(vecdeque.len(), sample.len() - samples_per_op);

        // dbg!(&vecdeque);

        recvbuf_to_vecdec(&send_recv_buf, &mut vecdeque);
        assert_eq!(vecdeque.len(), sample.len());

        // dbg!(&vecdeque);
    }

    let result: Vec<f32> = vecdeque.into();
    assert!(sample == result);
}
