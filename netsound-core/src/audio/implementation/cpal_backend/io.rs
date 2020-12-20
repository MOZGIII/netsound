use super::CompatibleSample;
use crate::io::{AsyncReadItems, AsyncReadItemsExt, AsyncWriteItems, AsyncWriteItemsExt, WaitMode};
use futures::executor::block_on;

pub fn capture<'a, S, W>(from: &'a mut cpal::UnknownTypeInputBuffer<'a>, to: &mut W)
where
    S: CompatibleSample + 'a,
    W: AsyncWriteItems<S> + Unpin,
{
    let from = S::unwrap_cpal_input_buffer(from);
    let result = block_on(to.write_items(from, WaitMode::NoWait));
    let _ = result.expect("failed to write to shared buf");
}

pub fn play<'a, S, R>(from: &mut R, to: &'a mut cpal::UnknownTypeOutputBuffer<'a>)
where
    S: CompatibleSample + 'a,
    R: AsyncReadItems<S> + Unpin,
{
    let to = S::unwrap_cpal_output_buffer(to);
    let result = block_on(from.read_items(to, WaitMode::NoWait));
    let samples_read = result.expect("failed to read from shared buf");

    // We _must_ fill the whole `to` buffer.
    for sample_slot in to[samples_read..].iter_mut() {
        *sample_slot = S::EQUILIBRIUM;
    }
}
