use super::*;
use crate::io::*;
use futures::executor::block_on;
use futures::future::FutureExt;
use futures::pin_mut;
use futures::task::{Context, Poll};
use futures_test::task::{new_count_waker, panic_context};
use std::thread;
use std::time::Duration;

#[test]
fn test_sequence_write_read() {
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
fn test_read_non_pending_underflowing() {
    // Ensure VecDeque has expected capacity.
    let vd = VecDeque::from(vec![1, 2, 3]);
    assert_eq!(vd.capacity(), 3);

    let (_writer, reader) = vec_deque_buffer(vd);
    pin_mut!(reader);

    let mut read_buf = [0u8; 1024];

    let cx = &mut panic_context();
    let poll_result = reader.poll_read_items(cx, &mut read_buf[..]);
    let result = match poll_result {
        Poll::Pending => panic!("not supposed to be pending"),
        Poll::Ready(val) => val,
    };
    assert_eq!(result.unwrap(), 3);
}

#[test]
fn test_read_non_pending_non_underflowing() {
    // Ensure VecDeque has expected capacity.
    let vd = VecDeque::from(vec![1, 2, 3]);
    assert_eq!(vd.capacity(), 3);

    let (_writer, reader) = vec_deque_buffer(vd);
    pin_mut!(reader);

    let mut read_buf = [0u8; 2];

    let cx = &mut panic_context();
    let poll_result = reader.poll_read_items(cx, &mut read_buf[..]);
    let result = match poll_result {
        Poll::Pending => panic!("not supposed to be pending"),
        Poll::Ready(val) => val,
    };
    assert_eq!(result.unwrap(), 2);
}

#[test]
fn test_read_pending() {
    // Ensure VecDeque has expected capacity.
    let vd = VecDeque::from(vec![]);
    assert_eq!(vd.capacity(), 1);

    let (_writer, reader) = vec_deque_buffer(vd);
    pin_mut!(reader);

    let mut read_buf = [0u8; 1];

    let cx = &mut panic_context();
    let poll_result = reader.poll_read_items(cx, &mut read_buf[..]);
    assert!(poll_result.is_pending());
}

#[test]
fn test_write_non_pending_non_overflowing() {
    // Ensure VecDeque::with_capacity works as expected.
    let vd = VecDeque::with_capacity(3);
    assert_eq!(vd.capacity(), 3);

    let (writer, _reader) = vec_deque_buffer(vd);
    pin_mut!(writer);

    let write_buf = [1, 2];

    let cx = &mut panic_context();
    let poll_result = writer.poll_write_items(cx, &write_buf[..]);
    let result = match poll_result {
        Poll::Pending => panic!("not supposed to be pending"),
        Poll::Ready(val) => val,
    };
    assert_eq!(result.unwrap(), 2);
}

#[test]
fn test_write_non_pending_overflowing() {
    // Ensure VecDeque::with_capacity works as expected.
    let vd = VecDeque::with_capacity(3);
    assert_eq!(vd.capacity(), 3);

    let (writer, _reader) = vec_deque_buffer(vd);
    pin_mut!(writer);

    let write_buf = [1, 2, 3, 4];

    let cx = &mut panic_context();
    let poll_result = writer.poll_write_items(cx, &write_buf[..]);
    let result = match poll_result {
        Poll::Pending => panic!("not supposed to be pending"),
        Poll::Ready(val) => val,
    };
    assert_eq!(result.unwrap(), 3);
}

#[test]
fn test_write_pending() {
    // Ensure VecDeque::with_capacity works as expected.
    let vd = VecDeque::from(vec![1, 2, 3]);
    assert_eq!(vd.capacity(), 3);

    let (writer, _reader) = vec_deque_buffer(vd);
    pin_mut!(writer);

    let write_buf = [4];

    let cx = &mut panic_context();

    let poll_result = writer.poll_write_items(cx, &write_buf[..]);
    assert!(poll_result.is_pending());
}

#[test]
fn test_wakers() {
    let (mut writer, mut reader) = vec_deque_buffer(VecDeque::from(vec![1, 2, 3, 4]));

    let mut read_buf = [0u8; 1024];

    let items_read = block_on(reader.read_items(&mut read_buf[..])).unwrap();
    assert_eq!(items_read, 4);

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        let items_written = block_on(writer.write_items(&[5, 6])).unwrap();
        assert_eq!(items_written, 2);

        thread::sleep(Duration::from_millis(100));
        let items_written = block_on(writer.write_items(&[7, 8])).unwrap();
        assert_eq!(items_written, 2);
    });

    let items_read = block_on(reader.read_items(&mut read_buf[..])).unwrap();
    assert_eq!(items_read, 2);
}

#[test]
fn test_poll_wakers_read() {
    // Ensure VecDeque has expected capacity.
    let vd = VecDeque::from(vec![1, 2, 3]);
    assert_eq!(vd.capacity(), 3);

    let (mut writer, mut reader) = vec_deque_buffer(vd);

    let mut read_buf = [0u8; 1024];

    let (waker, count) = new_count_waker();
    let mut cx = Context::from_waker(&waker);
    assert_eq!(count, 0);

    let items_read_poll = reader.read_items(&mut read_buf[..]).poll_unpin(&mut cx);
    assert!(items_read_poll.is_ready());
    assert_eq!(count, 0);
    let items_read = assert_extract_ready(items_read_poll).unwrap();
    assert_eq!(items_read, 3);

    let items_read_poll = reader.read_items(&mut read_buf[..]).poll_unpin(&mut cx);
    assert!(items_read_poll.is_pending());
    assert_eq!(count, 0);

    let items_written_poll = writer.write_items(&[4, 5]).poll_unpin(&mut cx);
    assert!(items_written_poll.is_ready());
    assert_eq!(count, 1);
    let items_written = assert_extract_ready(items_written_poll).unwrap();
    assert_eq!(items_written, 2);

    let items_read_poll = reader.read_items(&mut read_buf[..]).poll_unpin(&mut cx);
    assert!(items_read_poll.is_ready());
    assert_eq!(count, 1);
    let items_read = assert_extract_ready(items_read_poll).unwrap();
    assert_eq!(items_read, 2);
}

#[test]
fn test_poll_wakers_write() {
    // Ensure VecDeque has expected capacity.
    let vd = VecDeque::with_capacity(3);
    assert_eq!(vd.capacity(), 3);

    let (mut writer, mut reader) = vec_deque_buffer(vd);

    let mut read_buf = [0u8; 1024];

    let (waker, count) = new_count_waker();
    let mut cx = Context::from_waker(&waker);
    assert_eq!(count, 0);

    let items_written_poll = writer.write_items(&[1, 2]).poll_unpin(&mut cx);
    assert!(items_written_poll.is_ready());
    assert_eq!(count, 0);
    let items_written = assert_extract_ready(items_written_poll).unwrap();
    assert_eq!(items_written, 2);

    let items_written_poll = writer.write_items(&[3, 4]).poll_unpin(&mut cx);
    assert!(items_written_poll.is_ready());
    assert_eq!(count, 0);
    let items_written = assert_extract_ready(items_written_poll).unwrap();
    assert_eq!(items_written, 1);

    let items_written_poll = writer.write_items(&[5, 6]).poll_unpin(&mut cx);
    assert!(items_written_poll.is_pending());
    assert_eq!(count, 0);

    let items_read_poll = reader.read_items(&mut read_buf[..]).poll_unpin(&mut cx);
    assert!(items_read_poll.is_ready());
    assert_eq!(count, 1);
    let items_read = assert_extract_ready(items_read_poll).unwrap();
    assert_eq!(items_read, 3);

    let items_written_poll = writer.write_items(&[5, 6]).poll_unpin(&mut cx);
    assert!(items_written_poll.is_ready());
    assert_eq!(count, 1);
    let items_written = assert_extract_ready(items_written_poll).unwrap();
    assert_eq!(items_written, 2);

    let items_read_poll = reader.read_items(&mut read_buf[..]).poll_unpin(&mut cx);
    assert!(items_read_poll.is_ready());
    assert_eq!(count, 1);
    let items_read = assert_extract_ready(items_read_poll).unwrap();
    assert_eq!(items_read, 2);
}

fn assert_extract_ready<T>(poll: Poll<T>) -> T {
    assert!(poll.is_ready());
    match poll {
        Poll::Ready(val) => val,
        Poll::Pending => unreachable!(),
    }
}