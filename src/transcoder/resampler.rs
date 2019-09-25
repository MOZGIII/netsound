use super::*;
use crate::buf::VecDequeBuffer;
use crate::match_channels;
use crate::sample::Sample;
use crate::samples_filter::NormalizeChannelsExt;
use async_trait::async_trait;
use sample::{interpolate, Duplex};
use std::io::Result;
use std::pin::Pin;
use std::task::{Context, Poll};

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

impl<S: Sample> AsyncWriteItems<S> for Resampler<S> {
    fn poll_write_items(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        items: &[S],
    ) -> Poll<Result<usize>> {
        Pin::new(&mut self.from_buf).poll_write_items(cx, items)
    }
}

impl<S: Sample> AsyncReadItems<S> for Resampler<S> {
    fn poll_read_items(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        items: &mut [S],
    ) -> Poll<Result<usize>> {
        Pin::new(&mut self.to_buf).poll_read_items(cx, items)
    }
}

impl<S: Sample> AsyncItemsAvailable<S> for Resampler<S> {
    fn poll_items_available(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<usize>> {
        Pin::new(&mut self.to_buf).poll_items_available(cx)
    }
}

#[async_trait]
impl<S> Transcode<S, S> for Resampler<S>
where
    S: Sample + Duplex<f64> + Unpin,
{
    type Ok = ();
    type Error = std::io::Error;

    async fn transcode(&mut self) -> Result<Self::Ok> {
        let to_channels = self.to_channels;
        let this = &mut *self;

        match_channels! {
            F => [to_channels] => {
                use sample::{signal, Signal};

                let mut from_signal =
                    signal::from_interleaved_samples_iter::<_, F<S>>(this.from_buf.drain(..).normalize_channels(this.from_channels, to_channels));
                let interpolator = interpolate::Linear::from_source(&mut from_signal);
                let converter = interpolate::Converter::from_hz_to_hz(
                    from_signal,
                    interpolator,
                    this.from_hz,
                    this.to_hz,
                );

                use sample::Frame;
                this.to_buf.extend(
                    converter
                        .until_exhausted()
                        .flat_map(|frame| frame.channels()),
                );
            }
        }

        Ok(())
    }
}
