use super::*;
use crate::io::*;
use futures::executor::block_on;
use std::thread;
use std::time::Duration;

#[test]
fn test_write_read() {
    let (mut writer, mut reader) = vec_deque_buffer_with_capacity::<u8>(1024);

    let write_buf = [1, 2, 3, 4];
    let mut read_buf = [0u8; 1024];

    let items_written = block_on(writer.write_items(&write_buf[..])).unwrap();
    assert_eq!(items_written, 4);

    let items_read = block_on(reader.read_items(&mut read_buf)).unwrap();
    assert_eq!(items_read, 4);

    assert_eq!(&read_buf[..items_read], &write_buf[..]);
}

#[test]
fn test_read_waker() {
    let (mut writer, mut reader) = vec_deque_buffer(VecDeque::from(vec![1, 2, 3, 4]));

    let mut read_buf = [0u8; 1024];

    let items_read = block_on(reader.read_items(&mut read_buf[..])).unwrap();
    assert_eq!(items_read, 4);

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        writer.write_items(&[5, 6]);

        thread::sleep(Duration::from_millis(100));
        writer.write_items(&[7, 8]);
    });

    let items_read = block_on(reader.read_items(&mut read_buf[..])).unwrap();
    assert_eq!(items_read, 0);
}
