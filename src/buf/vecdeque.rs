use crate::io::{AsyncReadItems, AsyncWriteItems, WaitMode};
use crate::log::trace;
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

impl<T> Inner<T> {
    fn wake_reader_if_needed(&mut self) {
        let should_wake = !self.vd.is_empty();
        if should_wake {
            trace!("waking reader");
            self.read_waker.wake();
        }
    }

    fn wake_writer_if_needed(&mut self) {
        let should_wake = !is_full(&self.vd);
        if should_wake {
            trace!("waking writer");
            self.write_waker.wake();
        }
    }
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
        wait_mode: WaitMode,
    ) -> Poll<Result<usize>> {
        trace!("read: before lock");
        let mut inner = ready!(self.inner.poll_lock(cx));
        trace!("read: after lock");

        let vd = &mut inner.vd;

        if vd.is_empty() {
            return match wait_mode {
                WaitMode::WaitForReady => {
                    inner.read_waker.register(cx.waker());
                    trace!("read: return with pending");
                    Poll::Pending
                }
                WaitMode::NoWait => {
                    trace!("read: return with ready for no wait");
                    Poll::Ready(Ok(0))
                }
            };
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
        let len = vd.len();

        inner.wake_writer_if_needed();

        trace!("read: return with ready: {} filled, {} len", filled, len);
        Poll::Ready(Ok(filled))
    }
}

impl<T: Unpin + Copy> AsyncWriteItems<T> for VecDequeBufferWriter<T> {
    fn poll_write_items(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        items: &[T],
        wait_mode: WaitMode,
    ) -> Poll<Result<usize>> {
        trace!("write: before lock");
        let mut inner = ready!(self.inner.poll_lock(cx));
        trace!("write: after lock");
        let vd = &mut inner.vd;

        if is_full(&vd) {
            return match wait_mode {
                WaitMode::WaitForReady => {
                    inner.write_waker.register(cx.waker());
                    trace!("write: return with pending");
                    return Poll::Pending;
                }
                WaitMode::NoWait => {
                    trace!("write: return with ready for no wait");
                    Poll::Ready(Ok(0))
                }
            };
        }

        let free_slots = vd.capacity() - vd.len();
        trace!("write: free slots: {}", free_slots);

        let mut filled: usize = 0;
        for item in items.iter().take(free_slots) {
            vd.push_back(*item);
            filled += 1;
        }
        let len = vd.len();

        inner.wake_reader_if_needed();

        trace!("write: return with ready: filled {}, len {}", filled, len);
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
        self.inner_guard.wake_writer_if_needed();
        self.inner_guard.wake_reader_if_needed();
        trace!(
            "InnerVecDequeGuard: wakers triggered on Drop, len {}",
            self.inner_guard.vd.len()
        );
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
    pub fn lock(&mut self) -> InnerVecDequeAcquire<'_, T> {
        let inner_acquire = self.inner.lock();
        InnerVecDequeAcquire { inner_acquire }
    }
}

impl<T> VecDequeBufferWriter<T> {
    pub fn lock(&mut self) -> InnerVecDequeAcquire<'_, T> {
        let inner_acquire = self.inner.lock();
        InnerVecDequeAcquire { inner_acquire }
    }
}

#[cfg(test)]
mod tests;
