use crate::log::no_scopes::Logger;
use crate::pcm::{Sample, StreamConfig};
use crate::Error;

pub trait StreamConfigNegotiator<TCaptureSample, TPlaybackSample>
where
    TCaptureSample: Sample,
    TPlaybackSample: Sample,
{
    type Continuation;

    fn negotiate<'a>(
        self,
        requested_capture_stream_configs: &'a [StreamConfig<TCaptureSample>],
        requested_playback_stream_configs: &'a [StreamConfig<TPlaybackSample>],
        logger: Logger,
    ) -> Result<
        (
            NegotiatedStreamConfigs<TCaptureSample, TPlaybackSample>,
            Self::Continuation,
        ),
        Error,
    >;
}

#[derive(Clone, Debug)]
pub struct NegotiatedStreamConfigs<TCaptureSample, TPlaybackSample>
where
    TCaptureSample: Sample,
    TPlaybackSample: Sample,
{
    pub capture: StreamConfig<TCaptureSample>,
    pub playback: StreamConfig<TPlaybackSample>,
}

pub trait Builder {
    type Backend: Backend;
    fn build(self) -> Result<Self::Backend, Error>;
}

/// Backend's only responsibility is to manage the runtime.
pub trait Backend: Send + Sync {
    fn run(&mut self);
}
