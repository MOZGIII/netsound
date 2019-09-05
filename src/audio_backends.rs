use crate::audio::*;
use crate::io::{ReadItems, WriteItems};
use crate::Error;

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

pub fn build<'a, TCaptureDataWriter, TPlaybackDataReader>(
    backend_to_use: AudioBackendToUse,
    builder: BackendBuilder<'_, f32, f32, TCaptureDataWriter, TPlaybackDataReader>,
) -> Result<Box<dyn Backend<CaptureSample = f32, PlaybackSample = f32> + 'a>, Error>
where
    TCaptureDataWriter: WriteItems<f32> + Send + 'a,
    TPlaybackDataReader: ReadItems<f32> + Send + 'a,
{
    match backend_to_use {
        AudioBackendToUse::Cpal => build_cpal(builder),
        AudioBackendToUse::PulseSimple => build_pulse_simple(builder),
    }
}

fn build_cpal<'a, TCaptureDataWriter, TPlaybackDataReader>(
    builder: BackendBuilder<'_, f32, f32, TCaptureDataWriter, TPlaybackDataReader>,
) -> Result<Box<dyn Backend<CaptureSample = f32, PlaybackSample = f32> + 'a>, Error>
where
    TCaptureDataWriter: WriteItems<f32> + Send + 'a,
    TPlaybackDataReader: ReadItems<f32> + Send + 'a,
{
    BoxedBackendBuilderFor::<cpal_backend::Backend<f32, f32, TCaptureDataWriter, TPlaybackDataReader>>::build_boxed(builder)
}

#[cfg(feature = "pulse_simple_backend")]
fn build_pulse_simple<'a, TCaptureDataWriter, TPlaybackDataReader>(
    builder: BackendBuilder<'_, f32, f32, TCaptureDataWriter, TPlaybackDataReader>,
) -> Result<Box<dyn Backend<CaptureSample = f32, PlaybackSample = f32> + 'a>, Error>
where
    TCaptureDataWriter: WriteItems<f32> + Send + 'a,
    TPlaybackDataReader: ReadItems<f32> + Send + 'a,
{
    BoxedBackendBuilderFor::<
        pulse_simple_backend::Backend<f32, f32, TCaptureDataWriter, TPlaybackDataReader>,
    >::build_boxed(builder)
}

#[cfg(not(feature = "pulse_simple_backend"))]
fn build_pulse_simple<'a, TCaptureDataWriter, TPlaybackDataReader>(
    _builder: BackendBuilder<'_, f32, f32, TCaptureDataWriter, TPlaybackDataReader>,
) -> Result<Box<dyn Backend<CaptureSample = f32, PlaybackSample = f32> + 'a>, Error>
where
    TCaptureDataWriter: WriteItems<f32> + Send + 'a,
    TPlaybackDataReader: ReadItems<f32> + Send + 'a,
{
    unimplemented!()
}
