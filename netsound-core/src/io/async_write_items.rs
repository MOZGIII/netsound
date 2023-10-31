use super::WaitMode;
use std::io::Result;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::{Context, Poll};

pub trait AsyncWriteItems<T: Unpin> {
    fn poll_write_items(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        items: &[T],
        wait_mode: WaitMode,
    ) -> Poll<Result<usize>>;
}

macro_rules! deref_async_write_items {
    ($T:ty) => {
        fn poll_write_items(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            items: &[$T],
            wait_mode: WaitMode,
        ) -> Poll<Result<usize>> {
            Pin::new(&mut **self).poll_write_items(cx, items, wait_mode)
        }
    };
}

impl<I: Unpin, T: ?Sized + AsyncWriteItems<I> + Unpin> AsyncWriteItems<I> for Box<T> {
    deref_async_write_items!(I);
}

impl<I: Unpin, T: ?Sized + AsyncWriteItems<I> + Unpin> AsyncWriteItems<I> for &mut T {
    deref_async_write_items!(I);
}

impl<T: Unpin, P> AsyncWriteItems<T> for Pin<P>
where
    P: DerefMut + Unpin,
    <P as Deref>::Target: AsyncWriteItems<T>,
{
    fn poll_write_items(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        items: &[T],
        wait_mode: WaitMode,
    ) -> Poll<Result<usize>> {
        self.get_mut()
            .as_mut()
            .poll_write_items(cx, items, wait_mode)
    }
}
