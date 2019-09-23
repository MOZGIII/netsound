use super::Backend;
use crate::format::Format;
use crate::Error;
use sample::Sample;

pub trait FormatNegotiator<TCaptureSample, TPlaybackSample>
where
    TCaptureSample: Sample,
    TPlaybackSample: Sample,
{
    type Continuation;

    fn negotiate_formats<'a>(
        self,
        request_capture_formats: &'a [Format<TCaptureSample>],
        request_playback_formats: &'a [Format<TPlaybackSample>],
    ) -> Result<
        (
            NegotiatedFormats<TCaptureSample, TPlaybackSample>,
            Self::Continuation,
        ),
        Error,
    >;
}

#[derive(Clone)]
pub struct NegotiatedFormats<TCaptureSample, TPlaybackSample>
where
    TCaptureSample: Sample,
    TPlaybackSample: Sample,
{
    pub capture_format: Format<TCaptureSample>,
    pub playback_format: Format<TPlaybackSample>,
}

pub trait BackendBuilder {
    type Backend: Backend;
    fn build(self) -> Result<Self::Backend, Error>;
}
