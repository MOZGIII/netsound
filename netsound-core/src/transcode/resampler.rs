use super::Transcode;
use crate::buf::{VecDequeBufferReader, VecDequeBufferWriter};
use crate::io::{AsyncReadItemsExt, WaitMode};
use crate::log::trace;
use crate::match_channels;
use crate::pcm::{self, Sample};
use crate::samples_filter::NormalizeChannelsExt;
use async_trait::async_trait;
use dasp_frame::Frame;
use dasp_sample::Duplex;
use dasp_signal::{interpolate, Signal};

#[derive(Debug)]
pub struct Resampler<S: Sample> {
    pub from_channels: pcm::Channels,
    pub to_channels: pcm::Channels,

    pub from_hz: f64,
    pub to_hz: f64,

    pub from_buf: VecDequeBufferReader<S>,
    pub to_buf: VecDequeBufferWriter<S>,
}

impl<S: Sample> Resampler<S> {
    #[must_use]
    pub fn new(
        from_channels: pcm::Channels,
        to_channels: pcm::Channels,
        from_hz: f64,
        to_hz: f64,
        from_buf: VecDequeBufferReader<S>,
        to_buf: VecDequeBufferWriter<S>,
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

#[async_trait]
impl<S> Transcode for Resampler<S>
where
    S: Sample + Duplex<f64> + Unpin,
{
    type Ok = futures::never::Never;

    // We're using macros here that unwrap the same code 32 times. Clippy seems
    // to interpret this incorrectly. Disabling it as a palse-positive.
    #[allow(clippy::cognitive_complexity)]
    async fn transcode_loop(&mut self) -> Result<Self::Ok, crate::Error> {
        let to_channels = self.to_channels;
        let this = &mut *self;

        match_channels! {
            F => [to_channels] => {
                let mut first_frame_data = F::<S>::EQUILIBRIUM;
                loop {
                    trace!("Resampler: before read_exact_items");
                    this.from_buf.read_exact_items(&mut first_frame_data, WaitMode::WaitForReady).await?;
                    trace!("Resampler: after read_exact_items");

                    trace!("Resampler: before locks");
                    let mut from_buf = this.from_buf.lock().await;
                    let mut to_buf = this.to_buf.lock().await;
                    trace!("Resampler: locks taken");

                    let from_buf_size_before = from_buf.len() + first_frame_data.len();
                    let to_buf_size_before = to_buf.len();

                    let iter = first_frame_data.iter().copied();
                    let iter = iter.chain(from_buf.drain(..));
                    let iter = iter.normalize_channels(this.from_channels, to_channels);
                    let mut from_signal = dasp_signal::from_interleaved_samples_iter::<_, F<S>>(iter);
                    let left = from_signal.next();
                    let right = from_signal.next();
                    let interpolator = dasp_interpolate::linear::Linear::new(left, right);
                    let converter = interpolate::Converter::from_hz_to_hz(
                        from_signal,
                        interpolator,
                        this.from_hz,
                        this.to_hz,
                    );

                    // TODO: this extend may cause an unexpected `to_buf`
                    // capacity growth. We should provide a better API,
                    // involving waiting for write readiness.
                    to_buf.extend(
                        converter
                            .until_exhausted()
                            .flat_map(Frame::channels),
                    );

                    let from_buf_size_after = from_buf.len();
                    let to_buf_size_after = to_buf.len();

                    drop(to_buf);
                    drop(from_buf);
                    trace!("Resampler: after locks");

                    trace!("Resampler: {} -> {}  =>  {} -> {}", from_buf_size_before, to_buf_size_before, from_buf_size_after, to_buf_size_after);
                }
            }
        }
    }
}
