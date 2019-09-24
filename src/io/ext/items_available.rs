use super::*;
use futures::future::Future;
use futures::task::{Context, Poll};
use std::io;
use std::marker::PhantomData;
use std::pin::Pin;

#[derive(Debug)]
pub struct ItemsAvailable<'a, T, P: ?Sized + Unpin> {
    provider: &'a mut P,
    data_type: PhantomData<T>,
}

impl<T: Unpin, P: ?Sized + Unpin> Unpin for ItemsAvailable<'_, T, P> {}

impl<'a, T: Unpin, P: AsyncItemsAvailable<T> + ?Sized + Unpin> ItemsAvailable<'a, T, P> {
    pub(super) fn new(provider: &'a mut P) -> Self {
        Self {
            provider,
            data_type: PhantomData,
        }
    }
}

impl<T: Unpin, P: AsyncItemsAvailable<T> + ?Sized + Unpin> Future for ItemsAvailable<'_, T, P> {
    type Output = io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        Pin::new(&mut this.provider).poll_items_available(cx)
    }
}
