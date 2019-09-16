use super::*;
use crate::buf::VecDequeBuffer;
use crate::match_channels;
use crate::samples_filter::CutExtraChannelsExt;
use sample::{interpolate, Duplex, Sample};
use std::io::Result;

/// Resampler acts as writer, reader and transcoder.
#[derive(Debug)]
pub struct Resampler<S: Sample> {
    pub from_channels: usize,
    pub to_channels: usize,

    pub from_hz: f64,
    pub to_hz: f64,

    pub from_buf: VecDequeBuffer<S>,
    pub to_buf: VecDequeBuffer<S>,
}

impl<S: Sample> Resampler<S> {
    pub fn new(
        from_channels: usize,
        to_channels: usize,
        from_hz: f64,
        to_hz: f64,
        from_buf: VecDequeBuffer<S>,
        to_buf: VecDequeBuffer<S>,
    ) -> Self {
        Self {
            from_channels,
            to_channels,
            from_hz,
            to_hz,
            from_buf,
            to_buf,
        }
    }
}

impl<S: Sample> WriteItems<S> for Resampler<S> {
    fn write_items(&mut self, items: &[S]) -> Result<usize> {
        self.from_buf.write_items(items)
    }
}

impl<S: Sample> ReadItems<S> for Resampler<S> {
    fn read_items(&mut self, items: &mut [S]) -> Result<usize> {
        self.to_buf.read_items(items)
    }
}

impl<S: Sample> ItemsAvailable<S> for Resampler<S> {
    fn items_available(&self) -> Result<usize> {
        self.to_buf.items_available()
    }
}

impl<S: Sample> Transcode<S, S> for Resampler<S>
where
    S: Duplex<f64>,
{
    type Ok = ();
    type Error = std::io::Error;

    fn transcode(&mut self) -> Result<Self::Ok> {
        let to_channels = self.to_channels;

        match_channels! {
            F => [to_channels] => {
                use sample::{signal, Signal};

                let mut from_signal =
                    signal::from_interleaved_samples_iter::<_, F<S>>(self.from_buf.drain(..).cut_extra_channels(self.from_channels, to_channels));
                let interpolator = interpolate::Linear::from_source(&mut from_signal);
                let converter = interpolate::Converter::from_hz_to_hz(
                    from_signal,
                    interpolator,
                    self.from_hz,
                    self.to_hz,
                );

                use sample::Frame;
                self.to_buf.extend(
                    converter
                        .until_exhausted()
                        .flat_map(|frame| frame.channels()),
                );
            }
        }

        Ok(())
    }
}
