use crate::io::{AsyncReadItems, AsyncWriteItems};
use crate::log::*;
use futures::lock::{BiLock, BiLockAcquire, BiLockGuard};
use futures::ready;
use futures::task::AtomicWaker;
use std::collections::VecDeque;
use std::future::Future;
use std::io::Result;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug)]
struct Inner<T> {
    vd: VecDeque<T>,

    // Waits on read on buffer becoming non-empty.
    read_waker: AtomicWaker,

    // Waits on write for buffer becoming non-full.
    write_waker: AtomicWaker,
}

pub fn vec_deque_buffer_with_capacity<T>(
    capacity: usize,
) -> (VecDequeBufferWriter<T>, VecDequeBufferReader<T>) {
    vec_deque_buffer(VecDeque::with_capacity(capacity))
}

pub fn vec_deque_buffer<T>(vd: VecDeque<T>) -> (VecDequeBufferWriter<T>, VecDequeBufferReader<T>) {
    let (reader_inner, writer_inner) = BiLock::new(Inner {
        vd,
        read_waker: AtomicWaker::new(),
        write_waker: AtomicWaker::new(),
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
        trace!("read: before lock");
        let mut inner = ready!(self.inner.poll_lock(cx));
        trace!("read: after lock");

        let vd = &mut inner.vd;

        if vd.is_empty() {
            inner.read_waker.register(cx.waker());
            trace!("read: return with pending");
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
            inner.write_waker.wake();
        }

        trace!("read: return with ready");
        Poll::Ready(Ok(filled))
    }
}

impl<T: Unpin + Copy> AsyncWriteItems<T> for VecDequeBufferWriter<T> {
    fn poll_write_items(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        items: &[T],
    ) -> Poll<Result<usize>> {
        trace!("write: before lock");
        let mut inner = ready!(self.inner.poll_lock(cx));
        trace!("write: after lock");
        let vd = &mut inner.vd;

        if is_full(&vd) {
            inner.write_waker.register(cx.waker());
            trace!("write: return with pending");
            return Poll::Pending;
        }

        let free_slots = vd.capacity() - vd.len();
        trace!("write: free slots: {}", free_slots);

        let mut filled: usize = 0;
        for item in items.iter().take(free_slots) {
            vd.push_back(*item);
            filled += 1;
        }

        let wake_reader = !vd.is_empty();
        if wake_reader {
            inner.read_waker.wake();
        }

        trace!("write: return with ready");
        Poll::Ready(Ok(filled))
    }
}

pub struct InnerVecDequeGuard<'a, T: Unpin> {
    inner_guard: BiLockGuard<'a, Inner<T>>,
}

impl<T: Unpin> Deref for InnerVecDequeGuard<'_, T> {
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

impl<T: Unpin> Drop for InnerVecDequeGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        // On lock drop, since we don't know what happened, we have to notify
        // all our wakers again, assuminbg they can now progress.

        if let Some(waker) = self.inner_guard.read_waker.take() {
            waker.wake();
        }
        if let Some(waker) = self.inner_guard.write_waker.take() {
            waker.wake();
        }

        trace!("InnerVecDequeGuard: wakers triggered on Drop");
    }
}

pub struct InnerVecDequeAcquire<'a, T> {
    inner_acquire: BiLockAcquire<'a, Inner<T>>,
}

impl<T> Unpin for InnerVecDequeAcquire<'_, T> {}

impl<'a, T: Unpin> Future for InnerVecDequeAcquire<'a, T> {
    type Output = InnerVecDequeGuard<'a, T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        trace!("InnerVecDequeAcquire: before lock");
        let inner_guard = ready!(Pin::new(&mut self.inner_acquire).poll(cx));
        trace!("InnerVecDequeAcquire: after lock");
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
