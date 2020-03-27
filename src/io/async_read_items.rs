use super::WaitMode;
use std::io::Result;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::{Context, Poll};

pub trait AsyncReadItems<T: Unpin> {
    fn poll_read_items(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        items: &mut [T],
        wait_mode: WaitMode,
    ) -> Poll<Result<usize>>;
}

macro_rules! deref_async_read_items {
    ($T:ty) => {
        fn poll_read_items(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            items: &mut [$T],
            wait_mode: WaitMode,
        ) -> Poll<Result<usize>> {
            Pin::new(&mut **self).poll_read_items(cx, items, wait_mode)
        }
    };
}

impl<T: Unpin, A: ?Sized + AsyncReadItems<T> + Unpin> AsyncReadItems<T> for Box<A> {
    deref_async_read_items!(T);
}

impl<T: Unpin, A: ?Sized + AsyncReadItems<T> + Unpin> AsyncReadItems<T> for &mut A {
    deref_async_read_items!(T);
}

impl<T: Unpin, P> AsyncReadItems<T> for Pin<P>
where
    P: DerefMut + Unpin,
    <P as Deref>::Target: AsyncReadItems<T>,
{
    fn poll_read_items(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        items: &mut [T],
        wait_mode: WaitMode,
    ) -> Poll<Result<usize>> {
        self.get_mut()
            .as_mut()
            .poll_read_items(cx, items, wait_mode)
    }
}
