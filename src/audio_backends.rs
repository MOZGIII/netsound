use crate::audio;
use crate::format::Format;
use crate::io::{ReadItems, WriteItems};
use crate::sync::Synced;
use sample::Sample;

#[derive(Debug)]
pub enum AudioBackendToUse {
    Cpal,
    PulseSimple,
}

impl AudioBackendToUse {
    pub fn from_env() -> Result<Self, std::env::VarError> {
        Ok(match std::env::var("AUDIO_BACKEND") {
            Ok(ref val) if val == "cpal" => AudioBackendToUse::Cpal,
            Ok(ref val) if val == "pulse_simple" => AudioBackendToUse::PulseSimple,
            // Defaults.
            Ok(_) | Err(std::env::VarError::NotPresent) => AudioBackendToUse::Cpal,
            // Invalid value.
            Err(e) => return Err(e),
        })
    }
}

pub fn build<
    'a,
    TCaptureData,
    TPlaybackData,
    TSharedCaptureDataBuilder,
    TSharedPlaybackDataBuilder,
>(
    backend_to_use: AudioBackendToUse,
    build_params: audio::build_flow::BuildParams<
        'a,
        f32,
        f32,
        TCaptureData,
        TPlaybackData,
        TSharedCaptureDataBuilder,
        TSharedPlaybackDataBuilder,
    >,
) -> Result<
    audio::build_flow::BuiltState<f32, f32, Synced<TCaptureData>, Synced<TPlaybackData>>,
    crate::Error,
>
where
    TCaptureData: WriteItems<f32> + Send + 'static,
    TPlaybackData: ReadItems<f32> + Send + 'static,

    TSharedCaptureDataBuilder: FnOnce(Format<f32>) -> Result<Synced<TCaptureData>, crate::Error>,
    TSharedPlaybackDataBuilder: FnOnce(Format<f32>) -> Result<Synced<TPlaybackData>, crate::Error>,
{
    match backend_to_use {
        AudioBackendToUse::Cpal => build_cpal(build_params),
        AudioBackendToUse::PulseSimple => build_pulse_simple(build_params),
    }
}

fn build_cpal<
    'a,
    TCaptureSample,
    TPlaybackSample,
    TCaptureData,
    TPlaybackData,
    TSharedCaptureDataBuilder,
    TSharedPlaybackDataBuilder,
>(
    build_params: audio::build_flow::BuildParams<
        'a,
        TCaptureSample,
        TPlaybackSample,
        TCaptureData,
        TPlaybackData,
        TSharedCaptureDataBuilder,
        TSharedPlaybackDataBuilder,
    >,
) -> Result<
    audio::build_flow::BuiltState<
        TCaptureSample,
        TPlaybackSample,
        Synced<TCaptureData>,
        Synced<TPlaybackData>,
    >,
    crate::Error,
>
where
    TCaptureSample: Sample + audio::cpal_backend::CompatibleSample + Send + Sync + 'static,
    TPlaybackSample: Sample + audio::cpal_backend::CompatibleSample + Send + Sync + 'static,

    TCaptureData: WriteItems<TCaptureSample> + Send + 'static,
    TPlaybackData: ReadItems<TPlaybackSample> + Send + 'static,

    TSharedCaptureDataBuilder:
        FnOnce(Format<TCaptureSample>) -> Result<Synced<TCaptureData>, crate::Error>,
    TSharedPlaybackDataBuilder:
        FnOnce(Format<TPlaybackSample>) -> Result<Synced<TPlaybackData>, crate::Error>,
{
    Ok(audio::build_flow::negotiate_and_build(
        audio::cpal_backend::FormatNegotiator,
        |continuation, shared_capture_data, shared_playback_data| {
            audio::cpal_backend::BackendBuilder {
                continuation,
                shared_capture_data_writer: shared_capture_data.clone(),
                shared_playback_data_reader: shared_playback_data.clone(),
            }
        },
        build_params,
    )?)
}

#[cfg(feature = "pulse_simple_backend")]
fn build_pulse_simple<
    'a,
    TCaptureSample,
    TPlaybackSample,
    TCaptureData,
    TPlaybackData,
    TSharedCaptureDataBuilder,
    TSharedPlaybackDataBuilder,
>(
    build_params: audio::build_flow::BuildParams<
        'a,
        TCaptureSample,
        TPlaybackSample,
        TCaptureData,
        TPlaybackData,
        TSharedCaptureDataBuilder,
        TSharedPlaybackDataBuilder,
    >,
) -> Result<
    audio::build_flow::BuiltState<
        TCaptureSample,
        TPlaybackSample,
        Synced<TCaptureData>,
        Synced<TPlaybackData>,
    >,
    crate::Error,
>
where
    TCaptureSample: Sample + audio::pulse_simple_backend::CompatibleSample + Send + Sync + 'static,
    TPlaybackSample: Sample + audio::pulse_simple_backend::CompatibleSample + Send + Sync + 'static,

    TCaptureData: WriteItems<TCaptureSample> + Send + 'static,
    TPlaybackData: ReadItems<TPlaybackSample> + Send + 'static,

    TSharedCaptureDataBuilder:
        FnOnce(Format<TCaptureSample>) -> Result<Synced<TCaptureData>, crate::Error>,
    TSharedPlaybackDataBuilder:
        FnOnce(Format<TPlaybackSample>) -> Result<Synced<TPlaybackData>, crate::Error>,
{
    Ok(audio::build_flow::negotiate_and_build(
        audio::pulse_simple_backend::FormatNegotiator,
        |continuation, shared_capture_data, shared_playback_data| {
            audio::pulse_simple_backend::BackendBuilder {
                continuation,
                shared_capture_data_writer: shared_capture_data.clone(),
                shared_playback_data_reader: shared_playback_data.clone(),
            }
        },
        build_params,
    )?)
}

#[cfg(not(feature = "pulse_simple_backend"))]
fn build_pulse_simple<
    'a,
    TCaptureSample,
    TPlaybackSample,
    TCaptureData,
    TPlaybackData,
    TSharedCaptureDataBuilder,
    TSharedPlaybackDataBuilder,
>(
    _build_params: audio::build_flow::BuildParams<
        'a,
        TCaptureSample,
        TPlaybackSample,
        TCaptureData,
        TPlaybackData,
        TSharedCaptureDataBuilder,
        TSharedPlaybackDataBuilder,
    >,
) -> Result<
    audio::build_flow::BuiltState<
        TCaptureSample,
        TPlaybackSample,
        Synced<TCaptureData>,
        Synced<TPlaybackData>,
    >,
    crate::Error,
>
where
    TCaptureSample: Sample,
    TPlaybackSample: Sample,

    TCaptureData: WriteItems<TCaptureSample> + Send + 'static,
    TPlaybackData: ReadItems<TPlaybackSample> + Send + 'static,

    TSharedCaptureDataBuilder:
        FnOnce(Format<TCaptureSample>) -> Result<Synced<TCaptureData>, crate::Error>,
    TSharedPlaybackDataBuilder:
        FnOnce(Format<TPlaybackSample>) -> Result<Synced<TPlaybackData>, crate::Error>,
{
    unimplemented!()
}
