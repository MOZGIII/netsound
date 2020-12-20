use super::{AsyncReadItems, WaitMode};
use futures::future::Future;
use futures::task::{Context, Poll};
use std::io;
use std::pin::Pin;

#[derive(Debug)]
pub struct ReadItems<'a, T, R: ?Sized + Unpin> {
    reader: &'a mut R,
    buf: &'a mut [T],
    wait_mode: WaitMode,
}

impl<T, R: ?Sized + Unpin> Unpin for ReadItems<'_, T, R> {}

impl<'a, T: Unpin, R: AsyncReadItems<T> + ?Sized + Unpin> ReadItems<'a, T, R> {
    pub(super) fn new(reader: &'a mut R, buf: &'a mut [T], wait_mode: WaitMode) -> Self {
        Self {
            reader,
            buf,
            wait_mode,
        }
    }
}

impl<T: Unpin, R: AsyncReadItems<T> + ?Sized + Unpin> Future for ReadItems<'_, T, R> {
    type Output = io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        Pin::new(&mut this.reader).poll_read_items(cx, this.buf, this.wait_mode)
    }
}
