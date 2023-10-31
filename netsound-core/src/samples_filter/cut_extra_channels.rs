#[derive(Debug)]
pub struct CutExtraChannels<I>
where
    I: Iterator,
{
    source_iter: I,
    source_channels: usize,
    target_channels: usize,
    current_channel: usize,
}

impl<I> CutExtraChannels<I>
where
    I: Iterator,
{
    /// # Panics
    ///
    /// Panics when the `source_channels` is less than `target_channels`.
    pub fn new(source_iter: I, source_channels: usize, target_channels: usize) -> Self {
        assert!(source_channels >= target_channels);
        Self {
            source_iter,
            source_channels,
            target_channels,
            current_channel: 0,
        }
    }
}

impl<I> Iterator for CutExtraChannels<I>
where
    I: Iterator,
{
    type Item = <I as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = match self.source_iter.next() {
                None => return None,
                Some(item) => item,
            };

            let return_current_item = self.current_channel < self.target_channels;
            self.current_channel += 1;

            let reset_current_channel = self.current_channel >= self.source_channels;

            if reset_current_channel {
                self.current_channel = 0;
            }

            if return_current_item {
                return Some(item);
            }
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub trait CutExtraChannelsExt: Iterator + Sized {
    fn cut_extra_channels(
        self,
        source_channels: usize,
        target_channels: usize,
    ) -> CutExtraChannels<Self> {
        CutExtraChannels::new(self, source_channels, target_channels)
    }
}

impl<I> CutExtraChannelsExt for I where I: Iterator + Sized {}

#[cfg(test)]
mod tests {
    use super::CutExtraChannelsExt;

    #[test]
    fn basic() {
        let result: Vec<_> = vec![1, 2, 3, 4]
            .into_iter()
            .cut_extra_channels(2, 1)
            .collect();
        assert_eq!(result, [1, 3]);
    }
}
