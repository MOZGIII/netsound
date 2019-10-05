use super::*;
use crate::sample::Sample;
use async_trait::async_trait;
use std::io::Result;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Noop acts as writer, reader and transcoder.
#[allow(dead_code)]
#[derive(Debug)]
pub struct Noop<S: Sample, T> {
    buf: T,
    sample_type: PhantomData<S>,
}

#[allow(dead_code)]
impl<S: Sample, T: AsyncWriteItems<S> + AsyncReadItems<S> + AsyncItemsAvailable<S>> Noop<S, T> {
    pub fn new(buf: T) -> Self {
        Self {
            buf,
            sample_type: PhantomData,
        }
    }
}

impl<S: Sample, T: AsyncWriteItems<S> + Unpin> AsyncWriteItems<S> for Noop<S, T> {
    fn poll_write_items(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        items: &[S],
    ) -> Poll<Result<usize>> {
        Pin::new(&mut self.buf).poll_write_items(cx, items)
    }
}

impl<S: Sample, T: AsyncReadItems<S> + Unpin> AsyncReadItems<S> for Noop<S, T> {
    fn poll_read_items(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        items: &mut [S],
    ) -> Poll<Result<usize>> {
        Pin::new(&mut self.buf).poll_read_items(cx, items)
    }
}

impl<S: Sample, T: AsyncItemsAvailable<S> + Unpin> AsyncItemsAvailable<S> for Noop<S, T> {
    fn poll_items_available(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<usize>> {
        Pin::new(&mut self.buf).poll_items_available(cx)
    }

    fn poll_wait_for_items_available(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        required_amount: usize,
    ) -> Poll<Result<usize>> {
        Pin::new(&mut self.buf).poll_wait_for_items_available(cx, required_amount)
    }
}

#[async_trait]
impl<S, T> Transcode for Noop<S, T>
where
    S: Sample + Unpin,
    T: AsyncWriteItems<S> + AsyncReadItems<S> + AsyncItemsAvailable<S> + Unpin + Send,
{
    type Ok = ();
    type Error = ();

    async fn transcode(&mut self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
