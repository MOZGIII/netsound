use super::CompatibleSample;
use futures::executor::block_on;
use netsound_core::io::{
    AsyncReadItems, AsyncReadItemsExt, AsyncWriteItems, AsyncWriteItemsExt, WaitMode,
};

pub fn capture<'a, S, W>(from: &'a [S], to: &mut W)
where
    S: CompatibleSample + 'a,
    W: AsyncWriteItems<S> + Unpin,
{
    let result = block_on(to.write_items(from, WaitMode::NoWait));
    let _ = result.expect("failed to write to shared buf");
}

pub fn play<'a, S, R>(from: &mut R, to: &'a mut [S])
where
    S: CompatibleSample + 'a,
    R: AsyncReadItems<S> + Unpin,
{
    let result = block_on(from.read_items(to, WaitMode::NoWait));
    let samples_read = result.expect("failed to read from shared buf");

    // We _must_ fill the whole `to` buffer.
    for sample_slot in to[samples_read..].iter_mut() {
        *sample_slot = S::EQUILIBRIUM;
    }
}
