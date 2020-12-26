use std::marker::PhantomData;

use crate::audio_backend::{self, Builder, StreamConfigNegotiator};
use crate::io::{AsyncReadItems, AsyncWriteItems};
use crate::log::Logger;
use crate::pcm::Sample;
use crate::pcm::StreamConfig;

#[derive(Clone, Copy)]
pub struct AudioBackendVariant<
    TCaptureSample: Sample,
    TPlaybackSample: Sample,
    const BACKEND_NAME: &'static str,
> {
    capture_sample_type: PhantomData<TCaptureSample>,
    playback_sample_type: PhantomData<TPlaybackSample>,
}

impl<TCaptureSample: Sample, TPlaybackSample: Sample, const BACKEND_NAME: &'static str>
    AudioBackendVariant<TCaptureSample, TPlaybackSample, BACKEND_NAME>
{
    pub const fn new() -> Self {
        Self {
            capture_sample_type: PhantomData,
            playback_sample_type: PhantomData,
        }
    }
}

impl<TCaptureSample: Sample, TPlaybackSample: Sample, const BACKEND_NAME: &'static str>
    std::fmt::Debug for AudioBackendVariant<TCaptureSample, TPlaybackSample, BACKEND_NAME>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::any::type_name;
        f.debug_struct("AudioBackendConfig")
            .field("backend_name", &BACKEND_NAME)
            .field("capture_sample_type", &type_name::<TCaptureSample>())
            .field("playback_sample_type", &type_name::<TPlaybackSample>())
            .finish()
    }
}

impl<TCaptureSample: Sample, TPlaybackSample: Sample, const BACKEND_NAME: &'static str>
    std::fmt::Display for AudioBackendVariant<TCaptureSample, TPlaybackSample, BACKEND_NAME>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::any::type_name;
        write!(
            f,
            "{}, {} capture, {} playback",
            BACKEND_NAME,
            type_name::<TCaptureSample>(),
            type_name::<TPlaybackSample>()
        )
    }
}

macro_rules! audio_backend_variants {
    (
        $vis:vis enum $enum_name:ident {
            $( [ $variant:ident, $name:literal, $capture_sample_type:ty, $playback_sample_type:ty ], ),*
        }
    ) => {
        #[derive(Clone, Copy, Debug)]
        $vis enum $enum_name {
            $(
                $variant(AudioBackendVariant<$capture_sample_type, $playback_sample_type, $name>),
            ),*
        }

        impl $enum_name {
            const ALL: &'static [Self] = &[
                $(
                    $enum_name::$variant(AudioBackendVariant::new()),
                ),*
            ];

            pub const fn all() -> &'static [Self] {
                Self::ALL
            }

            pub fn by_name(name: &str) -> Option<Self> {
                match name {
                    $(
                        $name => Some($enum_name::$variant(AudioBackendVariant::new())),
                    ),*
                    _ => None,
                }
            }
        }
    };
}

audio_backend_variants! {
    pub enum AnyAudioBackendVariant {
        [Cpal, "cpal", f32, f32],
    }
}

pub fn variant_from_env() -> Result<AnyAudioBackendVariant, anyhow::Error> {
    let name = match std::env::var("AUDIO_BACKEND") {
        Ok(name) => name,
        Err(std::env::VarError::NotPresent) => "cpal".to_owned(),
        Err(err) => return Err(err.into()),
    };
    Ok(AnyAudioBackendVariant::by_name(&name)
        .ok_or_else(|| anyhow::format_err!("audio backend {} is not available", name))?)
}

pub struct BuildParams<'a, TCaptureSample, TPlaybackSample>
where
    TCaptureSample: Sample,
    TPlaybackSample: Sample,
{
    pub request_capture_stream_configs: &'a [StreamConfig<TCaptureSample>],
    pub request_playback_stream_configs: &'a [StreamConfig<TPlaybackSample>],
    pub logger: Logger,
}

type NegotiateStreamConfigsContinuationFn<TCaptureDataWriter, TPlaybackDataReader> =
    dyn FnOnce(
        TCaptureDataWriter,
        TPlaybackDataReader,
    ) -> Result<Box<dyn audio_backend::Backend>, crate::Error>;

type NegotiateStreamConfigsResult<
    TCaptureSample,
    TPlaybackSample,
    TCaptureDataWriter,
    TPlaybackDataReader,
> = Result<
    (
        audio_backend::NegotiatedStreamConfigs<TCaptureSample, TPlaybackSample>,
        Box<NegotiateStreamConfigsContinuationFn<TCaptureDataWriter, TPlaybackDataReader>>,
    ),
    crate::Error,
>;

pub fn negotiate_stream_configs<TCaptureDataWriter, TPlaybackDataReader>(
    backend_to_use: AnyAudioBackendVariant,
    build_params: BuildParams<'_, f32, f32>,
) -> NegotiateStreamConfigsResult<f32, f32, TCaptureDataWriter, TPlaybackDataReader>
where
    TCaptureDataWriter: AsyncWriteItems<f32> + Unpin + Send + Sync + 'static,
    TPlaybackDataReader: AsyncReadItems<f32> + Unpin + Send + Sync + 'static,
{
    match backend_to_use {
        AnyAudioBackendVariant::Cpal(_) => build_cpal(build_params),
    }
}

fn build_cpal<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>(
    build_params: BuildParams<'_, TCaptureSample, TPlaybackSample>,
) -> NegotiateStreamConfigsResult<
    TCaptureSample,
    TPlaybackSample,
    TCaptureDataWriter,
    TPlaybackDataReader,
>
where
    TCaptureSample: Sample + netsound_audio_backend_cpal::CompatibleSample + Send + Sync + 'static,
    TPlaybackSample: Sample + netsound_audio_backend_cpal::CompatibleSample + Send + Sync + 'static,

    TCaptureDataWriter: AsyncWriteItems<TCaptureSample> + Unpin + Send + Sync + 'static,
    TPlaybackDataReader: AsyncReadItems<TPlaybackSample> + Unpin + Send + Sync + 'static,
{
    let stream_config_negotiator = netsound_audio_backend_cpal::StreamConfigNegotiator;
    let (negotiated_stream_configs, continuation) = stream_config_negotiator.negotiate(
        build_params.request_capture_stream_configs,
        build_params.request_playback_stream_configs,
        build_params.logger,
    )?;

    let continuation_adapter = move |capture_data_writer, playback_data_reader| {
        let backend = netsound_audio_backend_cpal::BackendBuilder {
            continuation,
            capture_data_writer,
            playback_data_reader,
        }
        .build()?;
        let backend: Box<dyn audio_backend::Backend> = Box::new(backend);
        Ok(backend)
    };
    let continuation_adapter = Box::new(continuation_adapter);

    Ok((negotiated_stream_configs, continuation_adapter))
}
