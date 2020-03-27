use crate::audio::{self, BackendBuilder, FormatNegotiator};
use crate::format::Format;
use crate::io::{AsyncReadItems, AsyncWriteItems};
use crate::log::Logger;
use crate::sample::Sample;

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

pub struct BuildParams<'a, TCaptureSample, TPlaybackSample>
where
    TCaptureSample: Sample,
    TPlaybackSample: Sample,
{
    pub request_capture_formats: &'a [Format<TCaptureSample>],
    pub request_playback_formats: &'a [Format<TPlaybackSample>],
    pub logger: Logger,
}

type NegotiateFormatsContinuationFn<TCaptureDataWriter, TPlaybackDataReader> =
    dyn FnOnce(
        TCaptureDataWriter,
        TPlaybackDataReader,
    ) -> Result<Box<dyn audio::Backend>, crate::Error>;

type NegotiateFormatsResult<
    TCaptureSample,
    TPlaybackSample,
    TCaptureDataWriter,
    TPlaybackDataReader,
> = Result<
    (
        audio::NegotiatedFormats<TCaptureSample, TPlaybackSample>,
        Box<NegotiateFormatsContinuationFn<TCaptureDataWriter, TPlaybackDataReader>>,
    ),
    crate::Error,
>;

pub fn negotiate_formats<TCaptureDataWriter, TPlaybackDataReader>(
    backend_to_use: &AudioBackendToUse,
    build_params: BuildParams<'_, f32, f32>,
) -> NegotiateFormatsResult<f32, f32, TCaptureDataWriter, TPlaybackDataReader>
where
    TCaptureDataWriter: AsyncWriteItems<f32> + Unpin + Send + Sync + 'static,
    TPlaybackDataReader: AsyncReadItems<f32> + Unpin + Send + Sync + 'static,
{
    match backend_to_use {
        AudioBackendToUse::Cpal => build_cpal(build_params),
        AudioBackendToUse::PulseSimple => build_pulse_simple(build_params),
    }
}

fn build_cpal<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>(
    build_params: BuildParams<'_, TCaptureSample, TPlaybackSample>,
) -> NegotiateFormatsResult<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>
where
    TCaptureSample: Sample + audio::cpal_backend::CompatibleSample + Send + Sync + 'static,
    TPlaybackSample: Sample + audio::cpal_backend::CompatibleSample + Send + Sync + 'static,

    TCaptureDataWriter: AsyncWriteItems<TCaptureSample> + Unpin + Send + Sync + 'static,
    TPlaybackDataReader: AsyncReadItems<TPlaybackSample> + Unpin + Send + Sync + 'static,
{
    let format_negotiator = audio::cpal_backend::FormatNegotiator;
    let (negotiated_formats, continuation) = format_negotiator.negotiate_formats(
        build_params.request_capture_formats,
        build_params.request_playback_formats,
        build_params.logger,
    )?;

    let continuation_adapter = move |capture_data_writer, playback_data_reader| {
        let backend = audio::cpal_backend::BackendBuilder {
            continuation,
            capture_data_writer,
            playback_data_reader,
        }
        .build()?;
        let backend: Box<dyn audio::Backend> = Box::new(backend);
        Ok(backend)
    };
    let continuation_adapter = Box::new(continuation_adapter);

    Ok((negotiated_formats, continuation_adapter))
}

#[cfg(feature = "pulse_simple_backend")]
fn build_pulse_simple<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>(
    build_params: BuildParams<'_, TCaptureSample, TPlaybackSample>,
) -> NegotiateFormatsResult<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>
where
    TCaptureSample: Sample + audio::pulse_simple_backend::CompatibleSample + Send + Sync + 'static,
    TPlaybackSample: Sample + audio::pulse_simple_backend::CompatibleSample + Send + Sync + 'static,

    TCaptureDataWriter: AsyncWriteItems<TCaptureSample> + Unpin + Send + Sync + 'static,
    TPlaybackDataReader: AsyncReadItems<TPlaybackSample> + Unpin + Send + Sync + 'static,
{
    let format_negotiator = audio::pulse_simple_backend::FormatNegotiator;
    let (negotiated_formats, continuation) = format_negotiator.negotiate_formats(
        build_params.request_capture_formats,
        build_params.request_playback_formats,
        build_params.logger,
    )?;

    let continuation_adapter = move |capture_data_writer, playback_data_reader| {
        let backend = audio::pulse_simple_backend::BackendBuilder {
            continuation,
            capture_data_writer,
            playback_data_reader,
        }
        .build()?;
        let backend: Box<dyn audio::Backend> = Box::new(backend);
        Ok(backend)
    };
    let continuation_adapter = Box::new(continuation_adapter);

    Ok((negotiated_formats, continuation_adapter))
}

#[cfg(not(feature = "pulse_simple_backend"))]
#[allow(clippy::needless_pass_by_value)]
fn build_pulse_simple<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>(
    _build_params: BuildParams<'_, TCaptureSample, TPlaybackSample>,
) -> NegotiateFormatsResult<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>
where
    TCaptureSample: Sample + Send + Sync + 'static,
    TPlaybackSample: Sample + Send + Sync + 'static,

    TCaptureDataWriter: AsyncWriteItems<TCaptureSample> + Unpin + Send + Sync + 'static,
    TPlaybackDataReader: AsyncReadItems<TPlaybackSample> + Unpin + Send + Sync + 'static,
{
    unimplemented!()
}
