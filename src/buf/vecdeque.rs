use crate::buf::{Buffer, FrameBuffer, SampleBuffer};
use crate::io::{AsyncItemsAvailable, AsyncReadItems, AsyncWriteItems};
use crate::sample::Sample;
use sample::Frame;
use std::collections::VecDeque;
use std::io::Result;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug, Clone)]
pub struct VecDequeBuffer<T>(VecDeque<T>);

impl<T> VecDequeBuffer<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self(VecDeque::with_capacity(capacity))
    }
}

impl<A> std::iter::FromIterator<A> for VecDequeBuffer<A> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = A>,
    {
        Self(VecDeque::from_iter(iter))
    }
}

impl<T> From<Vec<T>> for VecDequeBuffer<T> {
    fn from(vec: Vec<T>) -> Self {
        Self(VecDeque::from(vec))
    }
}

impl<T> Into<Vec<T>> for VecDequeBuffer<T> {
    fn into(self) -> Vec<T> {
        self.0.into()
    }
}

impl<T> std::ops::Deref for VecDequeBuffer<T> {
    type Target = VecDeque<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for VecDequeBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Unpin> AsyncReadItems<T> for VecDequeBuffer<T> {
    fn poll_read_items(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        items: &mut [T],
    ) -> Poll<Result<usize>> {
        let vd = &mut self.0;
        let mut filled: usize = 0;
        for item_slot in items.iter_mut() {
            match vd.pop_front() {
                None => break,
                Some(item) => {
                    *item_slot = item;
                    filled += 1;
                }
            }
        }
        Poll::Ready(Ok(filled))
    }
}

impl<T: Unpin + Copy> AsyncWriteItems<T> for VecDequeBuffer<T> {
    fn poll_write_items(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        items: &[T],
    ) -> Poll<Result<usize>> {
        let vd = &mut self.0;
        let mut filled: usize = 0;
        for item in items.iter() {
            vd.push_back(*item);
            filled += 1;
        }
        Poll::Ready(Ok(filled))
    }
}

impl<T: Unpin> AsyncItemsAvailable<T> for VecDequeBuffer<T> {
    fn poll_items_available(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<usize>> {
        Poll::Ready(Ok(self.0.len()))
    }
}

impl<T: Copy + Unpin + Send> Buffer for VecDequeBuffer<T> {
    type Item = T;
}

impl<S: Sample> SampleBuffer for VecDequeBuffer<S> {
    type Sample = S;
}

impl<F: Frame + Send + Unpin> FrameBuffer for VecDequeBuffer<F> {
    type Frame = F;
}
