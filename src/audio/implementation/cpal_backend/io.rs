use super::CompatibleSample;
use crate::io::{AsyncReadItems, AsyncReadItemsExt, AsyncWriteItems, AsyncWriteItemsExt};

pub async fn capture<'a, S, W>(from: &'a mut cpal::UnknownTypeInputBuffer<'a>, to: &mut W)
where
    S: CompatibleSample + 'a,
    W: AsyncWriteItems<S> + Unpin,
{
    to.write_items(S::unwrap_cpal_input_buffer(from))
        .await
        .expect("failed to write to shared buf");
}

pub async fn play<'a, S, R>(from: &mut R, to: &'a mut cpal::UnknownTypeOutputBuffer<'a>)
where
    S: CompatibleSample + 'a,
    R: AsyncReadItems<S> + Unpin,
{
    let to = S::unwrap_cpal_output_buffer(to);
    let samples_read = from
        .read_items(to)
        .await
        .expect("failed to read from shared buf");

    // We _must_ fill the whole `to` buffer.
    for sample_slot in to[samples_read..].iter_mut() {
        *sample_slot = S::equilibrium();
    }
}
