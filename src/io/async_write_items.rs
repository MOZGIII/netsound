use std::io::Result;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::{Context, Poll};

pub trait AsyncWriteItems<T: Unpin> {
    fn poll_write_items(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        items: &[T],
    ) -> Poll<Result<usize>>;
}

macro_rules! deref_async_write_items {
    ($T:ty) => {
        fn poll_write_items(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            items: &[$T],
        ) -> Poll<Result<usize>>
        {
            Pin::new(&mut **self).poll_write_items(cx, items)
        }
    }
}

impl<T: Unpin, A: ?Sized + AsyncWriteItems<T> + Unpin> AsyncWriteItems<T> for Box<A> {
    deref_async_write_items!(T);
}

impl<T: Unpin, A: ?Sized + AsyncWriteItems<T> + Unpin> AsyncWriteItems<T> for &mut A {
    deref_async_write_items!(T);
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
    ) -> Poll<Result<usize>> {
        self.get_mut().as_mut().poll_write_items(cx, items)
    }
}
