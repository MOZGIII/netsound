use super::*;
use futures::future::Future;
use futures::task::{Context, Poll};
use std::io;
use std::marker::PhantomData;
use std::pin::Pin;

#[derive(Debug)]
pub struct WaitForItemsAvailable<'a, T, P: ?Sized + Unpin> {
    provider: &'a mut P,
    required_items: usize,
    data_type: PhantomData<T>,
}

impl<T: Unpin, P: ?Sized + Unpin> Unpin for WaitForItemsAvailable<'_, T, P> {}

impl<'a, T: Unpin, P: AsyncItemsAvailable<T> + ?Sized + Unpin> WaitForItemsAvailable<'a, T, P> {
    pub(super) fn new(provider: &'a mut P, required_items: usize) -> Self {
        Self {
            provider,
            required_items,
            data_type: PhantomData,
        }
    }
}

impl<T: Unpin, P: AsyncItemsAvailable<T> + ?Sized + Unpin> Future
    for WaitForItemsAvailable<'_, T, P>
{
    type Output = io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        let required_items = this.required_items;
        Pin::new(&mut this.provider).poll_wait_for_items_available(cx, required_items)
    }
}
