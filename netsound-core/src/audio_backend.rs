use crate::log::no_scopes::Logger;
use crate::pcm::{Sample, StreamConfig};

use async_trait::async_trait;

pub trait StreamConfigNegotiator<TCaptureSample, TPlaybackSample>
where
    TCaptureSample: Sample,
    TPlaybackSample: Sample,
{
    type Continuation;
    type Error;

    #[allow(clippy::type_complexity)]
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
        Self::Error,
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
    type Error;
    fn build(self) -> Result<Self::Backend, Self::Error>;
}

/// Backend's only responsibility is to manage the runtime.
#[async_trait]
pub trait Backend: Send + Sync {
    async fn run(&mut self);
}
