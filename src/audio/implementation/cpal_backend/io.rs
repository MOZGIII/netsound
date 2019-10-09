use super::CompatibleSample;
use crate::io::{AsyncReadItems, AsyncReadItemsExt, AsyncWriteItems, AsyncWriteItemsExt};
use crate::log::*;

pub fn capture<'a, S, W>(from: &'a mut cpal::UnknownTypeInputBuffer<'a>, to: &mut W)
where
    S: CompatibleSample + 'a,
    W: AsyncWriteItems<S> + Unpin,
{
    let from = S::unwrap_cpal_input_buffer(from);
    let result = match attempt_async(to.write_items(from)) {
        None => {
            trace!("Capture: unable to write to shared buffer");
            return;
        }
        Some(val) => val,
    };
    trace!("Captrue: write to shared buffer ok");
    let _ = result.expect("failed to write to shared buf");
}

pub fn play<'a, S, R>(from: &mut R, to: &'a mut cpal::UnknownTypeOutputBuffer<'a>)
where
    S: CompatibleSample + 'a,
    R: AsyncReadItems<S> + Unpin,
{
    let to = S::unwrap_cpal_output_buffer(to);
    let result = match attempt_async(from.read_items(to)) {
        None => {
            trace!("Playback: unable to read from shared buffer");
            return;
        }
        Some(val) => val,
    };
    trace!("Playback: read to shared buffer ok");
    let samples_read = result.expect("failed to read from shared buf");

    // We _must_ fill the whole `to` buffer.
    for sample_slot in to[samples_read..].iter_mut() {
        *sample_slot = S::equilibrium();
    }
}

fn attempt_async<F>(mut f: F) -> Option<F::Output>
where
    F: std::future::Future + Unpin,
{
    use futures::{task::noop_waker, FutureExt};
    use std::task::{Context, Poll};

    let mut waker = noop_waker();
    let mut cx = Context::from_waker(&mut waker);
    match f.poll_unpin(&mut cx) {
        Poll::Pending => None,
        Poll::Ready(val) => Some(val),
    }
}
