use super::Backend;
use crate::format::Format;
use crate::sync::Synced;
use crate::Error;
use sample::Sample;

pub struct Builder<
    'a,
    TCaptureSample,
    TPlaybackSample,
    TCaptureData,
    TPlaybackData,
    TSharedCaptureDataBuilder,
    TSharedPlaybackDataBuilder,
> where
    TCaptureSample: Sample,
    TPlaybackSample: Sample,

    TSharedCaptureDataBuilder:
        FnOnce(Format<TCaptureSample>) -> Result<Synced<TCaptureData>, crate::Error>,
    TSharedPlaybackDataBuilder:
        FnOnce(Format<TPlaybackSample>) -> Result<Synced<TPlaybackData>, crate::Error>,
{
    pub request_capture_formats: &'a [Format<TCaptureSample>],
    pub request_playback_formats: &'a [Format<TPlaybackSample>],

    pub shared_capture_data_builder: TSharedCaptureDataBuilder,
    pub shared_playback_data_builder: TSharedPlaybackDataBuilder,
}

pub trait Build<
    TBackend,
    TCaptureSample,
    TPlaybackSample,
    TCaptureData,
    TPlaybackData,
    TSharedCaptureDataBuilder,
    TSharedPlaybackDataBuilder,
> where
    TBackend: Backend,

    TCaptureSample: Sample,
    TPlaybackSample: Sample,

    TSharedCaptureDataBuilder:
        FnOnce(Format<TCaptureSample>) -> Result<Synced<TCaptureData>, Error>,
    TSharedPlaybackDataBuilder:
        FnOnce(Format<TPlaybackSample>) -> Result<Synced<TPlaybackData>, Error>,
{
    fn build(
        self,
    ) -> Result<
        BuiltState<
            TBackend,
            TCaptureSample,
            TPlaybackSample,
            Synced<TCaptureData>,
            Synced<TPlaybackData>,
        >,
        Error,
    >;
}

pub struct BuiltState<
    TBackend,
    TCaptureSample: Sample,
    TPlaybackSample: Sample,
    TSharedCaptureData,
    TSharedPlaybackData,
> {
    pub backend: TBackend,
    pub capture_format: Format<TCaptureSample>,
    pub playback_format: Format<TPlaybackSample>,
    pub shared_capture_data: TSharedCaptureData,
    pub shared_playback_data: TSharedPlaybackData,
}

impl<
        'a,
        TBackend: Backend + 'a,
        TCaptureSample: Sample + 'a,
        TPlaybackSample: Sample + 'a,
        TSharedCaptureData: 'a,
        TSharedPlaybackData: 'a,
    >
    BuiltState<TBackend, TCaptureSample, TPlaybackSample, TSharedCaptureData, TSharedPlaybackData>
{
    pub fn into_dyn_backend(
        self,
    ) -> BuiltState<
        Box<dyn Backend + 'a>,
        TCaptureSample,
        TPlaybackSample,
        TSharedCaptureData,
        TSharedPlaybackData,
    > {
        let backend = self.backend;
        let backend = Box::new(backend);
        BuiltState {
            backend,
            capture_format: self.capture_format,
            playback_format: self.playback_format,
            shared_capture_data: self.shared_capture_data,
            shared_playback_data: self.shared_playback_data,
        }
    }
}
