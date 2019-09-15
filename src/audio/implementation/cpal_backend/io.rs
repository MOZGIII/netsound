use super::conv::{CpalInputConverter, CpalOutputConverter};
use crate::io::{ReadItems, WriteItems};
use sample::Sample;

pub fn capture<'a, S, W, C>(
    converter: &C,
    from: &'a mut cpal::UnknownTypeInputBuffer<'a>,
    to: &mut W,
) where
    S: Sample + 'a,
    W: WriteItems<S>,
    C: CpalInputConverter<Sample = S>,
{
    to.write_items(converter.convert(from))
        .expect("failed to write to shared buf");
}

pub fn play<'a, S, R, C>(converter: &C, from: &mut R, to: &'a mut cpal::UnknownTypeOutputBuffer<'a>)
where
    S: Sample + 'a,
    R: ReadItems<S>,
    C: CpalOutputConverter<Sample = S>,
{
    let to = converter.convert(to);
    let samples_read = from.read_items(to).expect("failed to read from shared buf");

    // We _must_ fill the whole `to` buffer.
    for sample_slot in to[samples_read..].iter_mut() {
        *sample_slot = S::equilibrium();
    }
}
