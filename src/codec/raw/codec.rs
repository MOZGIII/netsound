use crate::io::{AsyncReadItems, AsyncReadItemsExt, AsyncWriteItems, AsyncWriteItemsExt, WaitMode};
use byteorder::ByteOrder;
use std::io::Result;

pub async fn encode<E, T>(input: &mut T, output: &mut [u8]) -> Result<usize>
where
    E: ByteOrder,
    T: AsyncReadItems<f32> + Unpin,
{
    // TODO: implement more efficiently.

    // Get the amount of samples to read. Must be round, otherwise we can't
    // fit all the samples we read to the output.
    let samples_to_read = output.len() / 4;
    assert_eq!(samples_to_read * 4, output.len());

    let mut samples = Vec::with_capacity(samples_to_read);
    samples.resize(samples_to_read, 0_f32);

    let samples_read = input
        .read_items(&mut samples, WaitMode::WaitForReady)
        .await?;

    for (mut chunk, &sample) in output.chunks_exact_mut(4).zip(&samples[..samples_read]) {
        E::write_f32(&mut chunk, sample);
    }

    Ok(samples_read * 4)
}

pub async fn decode<E, T>(input: &[u8], output: &mut T) -> Result<usize>
where
    E: ByteOrder,
    T: AsyncWriteItems<f32> + Unpin,
{
    // TODO: implement more efficiently.

    // Get the amount of samples to write. Must be round, otherwise we can't
    // prepare the samples buffer for all bytes we have at the input.
    let samples_to_write = input.len() / 4;
    assert_eq!(samples_to_write * 4, input.len());

    let mut samples = Vec::with_capacity(samples_to_write);
    samples.resize(samples_to_write, 0_f32);

    for (chunk, sample_slot) in input.chunks(4).zip(&mut samples) {
        *sample_slot = E::read_f32(&chunk);
    }

    output.write_items(&samples, WaitMode::WaitForReady).await
}

// TODO: restore tests, see git history.
