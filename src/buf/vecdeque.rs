use crate::io::{AsyncReadItems, AsyncWriteItems};
use futures::lock::{BiLock, BiLockAcquire, BiLockGuard};
use futures::ready;
use std::collections::VecDeque;
use std::future::Future;
use std::io::Result;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

#[derive(Debug)]
struct Inner<T> {
    vd: VecDeque<T>,

    // Waits on read on buffer becoming non-empty.
    read_waker: Option<Waker>,

    // Waits on write for buffer becoming non-full.
    write_waker: Option<Waker>,
}

pub fn vec_deque_buffer_with_capacity<T>(
    capacity: usize,
) -> (VecDequeBufferWriter<T>, VecDequeBufferReader<T>) {
    vec_deque_buffer(VecDeque::with_capacity(capacity))
}

pub fn vec_deque_buffer<T>(vd: VecDeque<T>) -> (VecDequeBufferWriter<T>, VecDequeBufferReader<T>) {
    let (reader_inner, writer_inner) = BiLock::new(Inner {
        vd,
        read_waker: None,
        write_waker: None,
    });
    let writer = VecDequeBufferWriter {
        inner: writer_inner,
    };
    let reader = VecDequeBufferReader {
        inner: reader_inner,
    };
    (writer, reader)
}

#[derive(Debug)]
pub struct VecDequeBufferReader<T> {
    inner: BiLock<Inner<T>>,
}

#[derive(Debug)]
pub struct VecDequeBufferWriter<T> {
    inner: BiLock<Inner<T>>,
}

fn is_full<T>(vd: &VecDeque<T>) -> bool {
    vd.len() >= vd.capacity()
}

impl<T: Unpin> AsyncReadItems<T> for VecDequeBufferReader<T> {
    fn poll_read_items(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        items: &mut [T],
    ) -> Poll<Result<usize>> {
        let mut inner = ready!(self.inner.poll_lock(cx));
        let vd = &mut inner.vd;

        if vd.is_empty() {
            assert!(inner.read_waker.is_none());
            inner.read_waker = Some(cx.waker().clone());
            return Poll::Pending;
        }

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

        let wake_writer = !is_full(vd);

        if wake_writer {
            if let Some(waker) = inner.write_waker.take() {
                waker.wake();
            }
        }

        Poll::Ready(Ok(filled))
    }
}

impl<T: Unpin + Copy> AsyncWriteItems<T> for VecDequeBufferWriter<T> {
    fn poll_write_items(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        items: &[T],
    ) -> Poll<Result<usize>> {
        let mut inner = ready!(self.inner.poll_lock(cx));
        let vd = &mut inner.vd;

        if is_full(&vd) {
            assert!(inner.write_waker.is_none());
            inner.write_waker = Some(cx.waker().clone());
            return Poll::Pending;
        }

        let free_slots = dbg!(vd.capacity()) - dbg!(vd.len());

        let mut filled: usize = 0;
        for item in items.iter().take(free_slots) {
            vd.push_back(*item);
            filled += 1;
        }

        let wake_reader = vd.is_empty();

        if wake_reader {
            if let Some(waker) = inner.read_waker.take() {
                waker.wake();
            }
        }

        Poll::Ready(Ok(filled))
    }
}

pub struct InnerVecDequeGuard<'a, T> {
    inner_guard: BiLockGuard<'a, Inner<T>>,
}

impl<T> Deref for InnerVecDequeGuard<'_, T> {
    type Target = VecDeque<T>;
    fn deref(&self) -> &VecDeque<T> {
        &self.inner_guard.vd
    }
}

impl<T: Unpin> DerefMut for InnerVecDequeGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut VecDeque<T> {
        &mut self.inner_guard.vd
    }
}

pub struct InnerVecDequeAcquire<'a, T> {
    inner_acquire: BiLockAcquire<'a, Inner<T>>,
}

impl<T> Unpin for InnerVecDequeAcquire<'_, T> {}

impl<'a, T> Future for InnerVecDequeAcquire<'a, T> {
    type Output = InnerVecDequeGuard<'a, T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let inner_guard = ready!(Pin::new(&mut self.inner_acquire).poll(cx));
        Poll::Ready(InnerVecDequeGuard { inner_guard })
    }
}

impl<T> VecDequeBufferReader<T> {
    pub fn lock<'a>(&'a mut self) -> InnerVecDequeAcquire<'a, T> {
        let inner_acquire = self.inner.lock();
        InnerVecDequeAcquire { inner_acquire }
    }
}

impl<T> VecDequeBufferWriter<T> {
    pub fn lock<'a>(&'a mut self) -> InnerVecDequeAcquire<'a, T> {
        let inner_acquire = self.inner.lock();
        InnerVecDequeAcquire { inner_acquire }
    }
}

#[cfg(test)]
mod tests;
