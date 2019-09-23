use crate::audio;
use crate::format::Format;
use crate::io::{ReadItems, WriteItems};
use crate::sync::Synced;
use sample::Sample;

pub struct BuildParams<
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

pub struct BuiltState<TCaptureSample, TPlaybackSample, TSharedCaptureData, TSharedPlaybackData>
where
    TCaptureSample: Sample,
    TPlaybackSample: Sample,
{
    pub backend: Box<dyn audio::Backend>,
    pub negotiated_formats: audio::NegotiatedFormats<TCaptureSample, TPlaybackSample>,
    pub shared_capture_data: TSharedCaptureData,
    pub shared_playback_data: TSharedPlaybackData,
}

pub fn negotiate_and_build<
    'a,
    TCaptureSample,
    TPlaybackSample,
    TFormatNegotiator,
    TBackendBuilder,
    TContinuationLogic,
    TCaptureData,
    TPlaybackData,
    TSharedCaptureDataBuilder,
    TSharedPlaybackDataBuilder,
>(
    format_negotiator: TFormatNegotiator,
    continuation_logic: TContinuationLogic,
    params: BuildParams<
        'a,
        TCaptureSample,
        TPlaybackSample,
        TCaptureData,
        TPlaybackData,
        TSharedCaptureDataBuilder,
        TSharedPlaybackDataBuilder,
    >,
) -> Result<
    BuiltState<TCaptureSample, TPlaybackSample, Synced<TCaptureData>, Synced<TPlaybackData>>,
    crate::Error,
>
where
    TCaptureSample: Sample,
    TPlaybackSample: Sample,

    TFormatNegotiator: audio::FormatNegotiator<TCaptureSample, TPlaybackSample>,
    TBackendBuilder: audio::BackendBuilder + 'static,
    TContinuationLogic: FnOnce(
        <TFormatNegotiator as audio::FormatNegotiator<TCaptureSample, TPlaybackSample>>::Continuation,
        &Synced<TCaptureData>,
        &Synced<TPlaybackData>,
    ) -> TBackendBuilder,

    TCaptureData: WriteItems<TCaptureSample> + Send + 'static,
    TPlaybackData: ReadItems<TPlaybackSample> + Send + 'static,

    TSharedCaptureDataBuilder:
        FnOnce(Format<TCaptureSample>) -> Result<Synced<TCaptureData>, crate::Error>,
    TSharedPlaybackDataBuilder:
        FnOnce(Format<TPlaybackSample>) -> Result<Synced<TPlaybackData>, crate::Error>,
{
    let (negotiated_formats, continuation) = format_negotiator.negotiate_formats(
        params.request_capture_formats,
        params.request_playback_formats,
    )?;

    let shared_capture_data_builder = params.shared_capture_data_builder;
    let shared_playback_data_builder = params.shared_playback_data_builder;

    let shared_capture_data = shared_capture_data_builder(negotiated_formats.capture_format)?;
    let shared_playback_data = shared_playback_data_builder(negotiated_formats.playback_format)?;

    let builder = continuation_logic(continuation, &shared_capture_data, &shared_playback_data);
    let backend = builder.build()?;
    let backend: Box<dyn audio::Backend + 'static> = Box::new(backend);

    Ok(BuiltState {
        backend,
        negotiated_formats,
        shared_capture_data,
        shared_playback_data,
    })
}
