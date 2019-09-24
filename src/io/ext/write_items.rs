use super::*;
use futures::future::Future;
use futures::task::{Context, Poll};
use std::io;
use std::pin::Pin;

pub struct WriteItems<'a, T, W: ?Sized + Unpin> {
    writer: &'a mut W,
    buf: &'a [T],
}

impl<T, W: ?Sized + Unpin> Unpin for WriteItems<'_, T, W> {}

impl<'a, T: Unpin, W: AsyncWriteItems<T> + ?Sized + Unpin> WriteItems<'a, T, W> {
    pub(super) fn new(writer: &'a mut W, buf: &'a [T]) -> Self {
        Self { writer, buf }
    }
}

impl<T: Unpin, W: AsyncWriteItems<T> + ?Sized + Unpin> Future for WriteItems<'_, T, W> {
    type Output = io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        Pin::new(&mut this.writer).poll_write_items(cx, this.buf)
    }
}
