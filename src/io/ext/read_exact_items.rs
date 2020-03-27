use super::{AsyncReadItems, WaitMode};
use futures::future::Future;
use futures::ready;
use futures::task::{Context, Poll};
use std::io;
use std::pin::Pin;

#[derive(Debug)]
pub struct ReadExactItems<'a, T, R: ?Sized> {
    reader: &'a mut R,
    buf: &'a mut [T],
    wait_mode: WaitMode,
}

impl<T, R: ?Sized + Unpin> Unpin for ReadExactItems<'_, T, R> {}

impl<'a, T: Unpin, R: AsyncReadItems<T> + ?Sized + Unpin> ReadExactItems<'a, T, R> {
    pub(super) fn new(reader: &'a mut R, buf: &'a mut [T], wait_mode: WaitMode) -> Self {
        Self {
            reader,
            buf,
            wait_mode,
        }
    }
}

impl<T: Unpin, R: AsyncReadItems<T> + ?Sized + Unpin> Future for ReadExactItems<'_, T, R> {
    type Output = io::Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        while !this.buf.is_empty() {
            let n =
                ready!(Pin::new(&mut this.reader).poll_read_items(cx, this.buf, this.wait_mode))?;
            {
                let (_, rest) = std::mem::replace(&mut this.buf, &mut []).split_at_mut(n);
                this.buf = rest;
            }
            if n == 0 {
                return Poll::Ready(Err(io::ErrorKind::UnexpectedEof.into()));
            }
        }
        Poll::Ready(Ok(()))
    }
}
