use std::io::Result;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::{Context, Poll};

pub trait AsyncItemsAvailable<T: Unpin> {
    fn poll_items_available(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<usize>>;

    fn poll_wait_for_items_available(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        required_amount: usize,
    ) -> Poll<Result<usize>>;
}

macro_rules! deref_async_items_available {
    ($T:ty) => {
        fn poll_items_available(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<usize>>
        {
            Pin::new(&mut **self).poll_items_available(cx)
        }

        fn poll_wait_for_items_available(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            required_amount: usize,
        ) -> Poll<Result<usize>>
        {
            Pin::new(&mut **self).poll_wait_for_items_available(cx, required_amount)
        }
    }
}

impl<T: Unpin, A: ?Sized + AsyncItemsAvailable<T> + Unpin> AsyncItemsAvailable<T> for Box<A> {
    deref_async_items_available!(T);
}

impl<T: Unpin, A: ?Sized + AsyncItemsAvailable<T> + Unpin> AsyncItemsAvailable<T> for &mut A {
    deref_async_items_available!(T);
}

impl<T: Unpin, P> AsyncItemsAvailable<T> for Pin<P>
where
    P: DerefMut + Unpin,
    <P as Deref>::Target: AsyncItemsAvailable<T>,
{
    fn poll_items_available(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<usize>> {
        self.get_mut().as_mut().poll_items_available(cx)
    }

    fn poll_wait_for_items_available(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        required_amount: usize,
    ) -> Poll<Result<usize>> {
        self.get_mut()
            .as_mut()
            .poll_wait_for_items_available(cx, required_amount)
    }
}
