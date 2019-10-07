use super::*;
use crate::buf::{VecDequeBufferReader, VecDequeBufferWriter};
use crate::match_channels;
use crate::sample::Sample;
use crate::samples_filter::NormalizeChannelsExt;
use async_trait::async_trait;
use sample::{interpolate, Duplex};

#[derive(Debug)]
pub struct Resampler<S: Sample> {
    pub from_channels: usize,
    pub to_channels: usize,

    pub from_hz: f64,
    pub to_hz: f64,

    pub from_buf: VecDequeBufferReader<S>,
    pub to_buf: VecDequeBufferWriter<S>,
}

impl<S: Sample> Resampler<S> {
    pub fn new(
        from_channels: usize,
        to_channels: usize,
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
    type Ok = ();

    async fn transcode(&mut self) -> Result<Self::Ok, crate::Error> {
        let to_channels = self.to_channels;
        let this = &mut *self;

        match_channels! {
            F => [to_channels] => {
                use sample::{signal, Signal};

                let mut from_buf = this.from_buf.lock().await;
                let mut to_buf = this.to_buf.lock().await;

                // let from_buf_size_before = from_buf.len();
                // let to_buf_size_before = to_buf.len();

                if from_buf.len() > 0 {
                    let mut from_signal =
                        signal::from_interleaved_samples_iter::<_, F<S>>(from_buf.drain(..).normalize_channels(this.from_channels, to_channels));

                    let interpolator = interpolate::Linear::from_source(&mut from_signal);
                    let converter = interpolate::Converter::from_hz_to_hz(
                        from_signal,
                        interpolator,
                        this.from_hz,
                        this.to_hz,
                    );

                    use sample::Frame;
                    to_buf.extend(
                        converter
                            .until_exhausted()
                            .flat_map(|frame| frame.channels()),
                    );
                }

                // let from_buf_size_after = from_buf.len();
                // let to_buf_size_after = to_buf.len();

                // println!("resampler done: {} -> {}  =>  {} -> {}", from_buf_size_before, to_buf_size_before, from_buf_size_after, to_buf_size_after);

                drop(to_buf);
                drop(from_buf);
            }
        }

        Ok(())
    }
}
