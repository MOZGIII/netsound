use crate::sample::Sample;

pub struct AddSilentChannels<I>
where
    I: Iterator,
{
    source_iter: I,
    source_channels: usize,
    target_channels: usize,
    current_channel: usize,
}

impl<I> AddSilentChannels<I>
where
    I: Iterator,
{
    pub fn new(source_iter: I, source_channels: usize, target_channels: usize) -> Self {
        assert!(source_channels <= target_channels);
        Self {
            source_iter,
            source_channels,
            target_channels,
            current_channel: 0,
        }
    }
}

impl<I> Iterator for AddSilentChannels<I>
where
    I: Iterator,
    I::Item: Sample,
{
    type Item = <I as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let item = if self.current_channel < self.source_channels {
            self.source_iter.next()
        } else {
            Some(<<I as Iterator>::Item as sample::Sample>::equilibrium())
        };

        self.current_channel += 1;
        if self.current_channel >= self.target_channels {
            self.current_channel = 0;
        }

        item
    }
}

pub trait AddSilentChannelsExt: Iterator + Sized
where
    Self::Item: Sample,
{
    fn add_silent_channels(
        self,
        source_channels: usize,
        target_channels: usize,
    ) -> AddSilentChannels<Self> {
        AddSilentChannels::new(self, source_channels, target_channels)
    }
}

impl<I> AddSilentChannelsExt for I
where
    I: Iterator + Sized,
    I::Item: Sample,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let result: Vec<_> = vec![1, 2, 3, 4]
            .into_iter()
            .add_silent_channels(2, 4)
            .collect();
        assert_eq!(result, [1, 2, 0, 0, 3, 4, 0, 0]);
    }
}
