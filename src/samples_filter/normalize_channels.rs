use super::*;
use crate::sample::Sample;

pub enum NormalizeChannels<I>
where
    I: Iterator,
{
    Expand(AddSilentChannels<I>),
    Reduce(CutExtraChannels<I>),
    Noop(I),
}

impl<I> NormalizeChannels<I>
where
    I: Iterator,
    I::Item: Sample,
{
    pub fn new(source_iter: I, source_channels: usize, target_channels: usize) -> Self {
        if source_channels > target_channels {
            Self::Reduce(source_iter.cut_extra_channels(source_channels, target_channels))
        } else if source_channels < target_channels {
            Self::Expand(source_iter.add_silent_channels(source_channels, target_channels))
        } else {
            Self::Noop(source_iter)
        }
    }
}

impl<I> Iterator for NormalizeChannels<I>
where
    I: Iterator,
    I::Item: Sample,
{
    type Item = <I as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            NormalizeChannels::Expand(iter) => iter.next(),
            NormalizeChannels::Reduce(iter) => iter.next(),
            NormalizeChannels::Noop(iter) => iter.next(),
        }
    }
}

pub trait NormalizeChannelsExt: Iterator + Sized
where
    Self::Item: Sample,
{
    fn normalize_channels(
        self,
        source_channels: usize,
        target_channels: usize,
    ) -> NormalizeChannels<Self> {
        NormalizeChannels::new(self, source_channels, target_channels)
    }
}

impl<I> NormalizeChannelsExt for I
where
    I: Iterator + Sized,
    I::Item: Sample,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduce() {
        let result: Vec<_> = vec![1, 2, 3, 4]
            .into_iter()
            .normalize_channels(2, 1)
            .collect();
        assert_eq!(result, [1, 3]);
    }

    #[test]
    fn expand() {
        let result: Vec<_> = vec![1, 2, 3, 4]
            .into_iter()
            .normalize_channels(2, 4)
            .collect();
        assert_eq!(result, [1, 2, 0, 0, 3, 4, 0, 0]);
    }

    #[test]
    fn noop() {
        let result: Vec<_> = vec![1, 2, 3, 4]
            .into_iter()
            .normalize_channels(2, 2)
            .collect();
        assert_eq!(result, [1, 2, 3, 4]);
    }
}
