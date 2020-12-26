use std::marker::PhantomData;

use crate::audio_backend::{self, Builder, StreamConfigNegotiator};
use crate::io::{AsyncReadItems, AsyncWriteItems};
use crate::log::Logger;
use crate::pcm::{self, Sample};

#[derive(Clone, Copy)]
pub struct AudioBackendVariant<TCaptureSample, TPlaybackSample, const BACKEND_NAME: &'static str>
where
    TCaptureSample: Sample,
    TPlaybackSample: Sample,
{
    capture_sample_type: PhantomData<TCaptureSample>,
    playback_sample_type: PhantomData<TPlaybackSample>,
}

impl<TCaptureSample, TPlaybackSample, const BACKEND_NAME: &'static str>
    AudioBackendVariant<TCaptureSample, TPlaybackSample, BACKEND_NAME>
where
    TCaptureSample: Sample,
    TPlaybackSample: Sample,
{
    pub const fn new() -> Self {
        Self {
            capture_sample_type: PhantomData,
            playback_sample_type: PhantomData,
        }
    }
}

pub trait Factory {
    type CaptureSample: Sample;
    type PlaybackSample: Sample;

    fn build<TCaptureDataWriter, TPlaybackDataReader>(
        &self,
        build_params: BuildParams<
            '_,
            (pcm::SampleRate, pcm::Channels),
            (pcm::SampleRate, pcm::Channels),
        >,
    ) -> NegotiateStreamConfigsResult<
        Self::CaptureSample,
        Self::PlaybackSample,
        TCaptureDataWriter,
        TPlaybackDataReader,
    >
    where
        TCaptureDataWriter: AsyncWriteItems<Self::CaptureSample> + Unpin + Send + Sync + 'static,
        TPlaybackDataReader: AsyncReadItems<Self::PlaybackSample> + Unpin + Send + Sync + 'static;
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
            #[allow(dead_code)]
            const ALL: &'static [Self] = &[
                $(
                    $enum_name::$variant(AudioBackendVariant::new()),
                ),*
            ];

            #[allow(dead_code)]
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

        impl std::fmt::Display for $enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        $enum_name::$variant(val) => val.fmt(f),
                    ),*
                }
            }
        }
    };
}

impl std::str::FromStr for AnyAudioBackendVariant {
    type Err = anyhow::Error;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Ok(AnyAudioBackendVariant::by_name(&name)
            .ok_or_else(|| anyhow::format_err!("audio backend {} is not available", name))?)
    }
}

pub struct BuildParams<'a, TCaptureParams, TPlaybackParams> {
    pub request_capture_params: &'a [TCaptureParams],
    pub request_playback_params: &'a [TPlaybackParams],
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

audio_backend_variants! {
    pub enum AnyAudioBackendVariant {
        [Cpal, "cpal", f32, f32],
    }
}

// A hack for now.
// Ideally we want to include this in the macro, but this would require
// rewriting the macro in a different form, so this is left as is for now.
impl Factory for AnyAudioBackendVariant {
    type CaptureSample = f32;
    type PlaybackSample = f32;

    fn build<TCaptureDataWriter, TPlaybackDataReader>(
        &self,
        build_params: BuildParams<
            '_,
            (pcm::SampleRate, pcm::Channels),
            (pcm::SampleRate, pcm::Channels),
        >,
    ) -> NegotiateStreamConfigsResult<f32, f32, TCaptureDataWriter, TPlaybackDataReader>
    where
        TCaptureDataWriter: AsyncWriteItems<f32> + Unpin + Send + Sync + 'static,
        TPlaybackDataReader: AsyncReadItems<f32> + Unpin + Send + Sync + 'static,
    {
        match self {
            AnyAudioBackendVariant::Cpal(variant) => variant.build(build_params),
        }
    }
}

impl<TCaptureSample, TPlaybackSample> Factory
    for AudioBackendVariant<TCaptureSample, TPlaybackSample, "cpal">
where
    TCaptureSample: Sample + netsound_audio_backend_cpal::CompatibleSample + Send + Sync + 'static,
    TPlaybackSample: Sample + netsound_audio_backend_cpal::CompatibleSample + Send + Sync + 'static,
{
    type CaptureSample = TCaptureSample;
    type PlaybackSample = TPlaybackSample;

    fn build<TCaptureDataWriter, TPlaybackDataReader>(
        &self,
        build_params: BuildParams<
            '_,
            (pcm::SampleRate, pcm::Channels),
            (pcm::SampleRate, pcm::Channels),
        >,
    ) -> NegotiateStreamConfigsResult<
        Self::CaptureSample,
        Self::PlaybackSample,
        TCaptureDataWriter,
        TPlaybackDataReader,
    >
    where
        TCaptureDataWriter: AsyncWriteItems<Self::CaptureSample> + Unpin + Send + Sync + 'static,
        TPlaybackDataReader: AsyncReadItems<Self::PlaybackSample> + Unpin + Send + Sync + 'static,
    {
        let BuildParams {
            request_capture_params,
            request_playback_params,
            logger,
        } = build_params;

        let stream_config_negotiator = netsound_audio_backend_cpal::StreamConfigNegotiator;

        let request_capture_stream_configs: Vec<_> = request_capture_params
            .iter()
            .map(|(sample_rate, channels)| pcm::StreamConfig::new(*sample_rate, *channels))
            .collect();
        let request_playback_stream_configs: Vec<_> = request_playback_params
            .iter()
            .map(|(sample_rate, channels)| pcm::StreamConfig::new(*sample_rate, *channels))
            .collect();

        let (negotiated_stream_configs, continuation) = stream_config_negotiator.negotiate(
            &request_capture_stream_configs,
            &request_playback_stream_configs,
            logger,
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
}
