use crate::audio;
use crate::format::Format;
use crate::io::{ReadItems, WriteItems};
use crate::sync::Synced;

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
    builder: audio::Builder<
        'a,
        f32,
        f32,
        TCaptureData,
        TPlaybackData,
        TSharedCaptureDataBuilder,
        TSharedPlaybackDataBuilder,
    >,
) -> Result<
    audio::BuiltState<
        Box<dyn audio::Backend>,
        f32,
        f32,
        Synced<TCaptureData>,
        Synced<TPlaybackData>,
    >,
    crate::Error,
>
where
    TCaptureData: WriteItems<f32> + Send + 'static,
    TPlaybackData: ReadItems<f32> + Send + 'static,

    TSharedCaptureDataBuilder: FnOnce(Format<f32>) -> Result<Synced<TCaptureData>, crate::Error>,
    TSharedPlaybackDataBuilder: FnOnce(Format<f32>) -> Result<Synced<TPlaybackData>, crate::Error>,
{
    match backend_to_use {
        AudioBackendToUse::Cpal => build_cpal(builder),
        AudioBackendToUse::PulseSimple => build_pulse_simple(builder),
    }
}

fn build_cpal<
    'a,
    TCaptureData,
    TPlaybackData,
    TSharedCaptureDataBuilder,
    TSharedPlaybackDataBuilder,
>(
    builder: audio::Builder<
        'a,
        f32,
        f32,
        TCaptureData,
        TPlaybackData,
        TSharedCaptureDataBuilder,
        TSharedPlaybackDataBuilder,
    >,
) -> Result<
    audio::BuiltState<
        Box<dyn audio::Backend>,
        f32,
        f32,
        Synced<TCaptureData>,
        Synced<TPlaybackData>,
    >,
    crate::Error,
>
where
    TCaptureData: WriteItems<f32> + Send + 'static,
    TPlaybackData: ReadItems<f32> + Send + 'static,

    TSharedCaptureDataBuilder: FnOnce(Format<f32>) -> Result<Synced<TCaptureData>, crate::Error>,
    TSharedPlaybackDataBuilder: FnOnce(Format<f32>) -> Result<Synced<TPlaybackData>, crate::Error>,
{
    Ok(audio::Build::<
        crate::audio::cpal_backend::Backend<f32, f32, TCaptureData, TPlaybackData>,
        f32,
        f32,
        TCaptureData,
        TPlaybackData,
        TSharedCaptureDataBuilder,
        TSharedPlaybackDataBuilder,
    >::build(builder)?
    .into_dyn_backend())
}

#[cfg(feature = "pulse_simple_backend")]
fn build_pulse_simple<
    'a,
    TCaptureData,
    TPlaybackData,
    TSharedCaptureDataBuilder,
    TSharedPlaybackDataBuilder,
>(
    builder: audio::Builder<
        'a,
        f32,
        f32,
        TCaptureData,
        TPlaybackData,
        TSharedCaptureDataBuilder,
        TSharedPlaybackDataBuilder,
    >,
) -> Result<
    audio::BuiltState<
        Box<dyn audio::Backend>,
        f32,
        f32,
        Synced<TCaptureData>,
        Synced<TPlaybackData>,
    >,
    crate::Error,
>
where
    TCaptureData: WriteItems<f32> + Send + 'static,
    TPlaybackData: ReadItems<f32> + Send + 'static,

    TSharedCaptureDataBuilder: FnOnce(Format<f32>) -> Result<Synced<TCaptureData>, crate::Error>,
    TSharedPlaybackDataBuilder: FnOnce(Format<f32>) -> Result<Synced<TPlaybackData>, crate::Error>,
{
    Ok(audio::Build::<
        crate::audio::pulse_simple_backend::Backend<f32, f32, TCaptureData, TPlaybackData>,
        f32,
        f32,
        TCaptureData,
        TPlaybackData,
        TSharedCaptureDataBuilder,
        TSharedPlaybackDataBuilder,
    >::build(builder)?
    .into_dyn_backend())
}

#[cfg(not(feature = "pulse_simple_backend"))]
fn build_pulse_simple<
    'a,
    TCaptureData,
    TPlaybackData,
    TSharedCaptureDataBuilder,
    TSharedPlaybackDataBuilder,
>(
    _builder: audio::Builder<
        'a,
        f32,
        f32,
        TCaptureData,
        TPlaybackData,
        TSharedCaptureDataBuilder,
        TSharedPlaybackDataBuilder,
    >,
) -> Result<
    audio::BuiltState<
        Box<dyn audio::Backend>,
        f32,
        f32,
        Synced<TCaptureData>,
        Synced<TPlaybackData>,
    >,
    crate::Error,
>
where
    TCaptureData: WriteItems<f32> + Send + 'static,
    TPlaybackData: ReadItems<f32> + Send + 'static,

    TSharedCaptureDataBuilder: FnOnce(Format<f32>) -> Result<Synced<TCaptureData>, crate::Error>,
    TSharedPlaybackDataBuilder: FnOnce(Format<f32>) -> Result<Synced<TPlaybackData>, crate::Error>,
{
    unimplemented!()
}
