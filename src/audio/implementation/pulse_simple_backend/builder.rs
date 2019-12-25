use super::*;
use crate::audio;
use crate::format::Format;
use crate::io::{AsyncReadItems, AsyncWriteItems};
use crate::log::no_scopes::*;
use libpulse_binding as pulse;
use std::marker::PhantomData;

use self::pulse::stream::Direction;

pub struct FormatNegotiator;

impl<TCaptureSample, TPlaybackSample> audio::FormatNegotiator<TCaptureSample, TPlaybackSample>
    for FormatNegotiator
where
    TCaptureSample: CompatibleSample,
    TPlaybackSample: CompatibleSample,
{
    type Continuation = FormatNegotiatonContinuation<TCaptureSample, TPlaybackSample>;

    fn negotiate_formats<'a>(
        self,
        _request_capture_formats: &'a [Format<TCaptureSample>],
        _request_playback_formats: &'a [Format<TPlaybackSample>],
        logger: Logger,
    ) -> Result<
        (
            audio::NegotiatedFormats<TCaptureSample, TPlaybackSample>,
            Self::Continuation,
        ),
        crate::Error,
    > {
        let capture_format = Format::new(2, 48000);
        let playback_format = Format::new(2, 48000);

        let negotiated_formats = audio::NegotiatedFormats {
            capture_format,
            playback_format,
        };
        let continuation = FormatNegotiatonContinuation {
            capture_format,
            playback_format,
            logger,
        };
        Ok((negotiated_formats, continuation))
    }
}

pub struct FormatNegotiatonContinuation<TCaptureSample, TPlaybackSample>
where
    TCaptureSample: CompatibleSample,
    TPlaybackSample: CompatibleSample,
{
    capture_format: Format<TCaptureSample>,
    playback_format: Format<TPlaybackSample>,
    logger: Logger,
}

#[allow(clippy::module_name_repetitions)]
pub struct BackendBuilder<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>
where
    TCaptureSample: CompatibleSample,
    TPlaybackSample: CompatibleSample,
{
    pub continuation: FormatNegotiatonContinuation<TCaptureSample, TPlaybackSample>,

    pub capture_data_writer: TCaptureDataWriter,
    pub playback_data_reader: TPlaybackDataReader,
}

impl<'a, TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>
    audio::BackendBuilder
    for BackendBuilder<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>
where
    TCaptureSample: CompatibleSample + Send + Sync,
    TPlaybackSample: CompatibleSample + Send + Sync,

    TCaptureDataWriter: AsyncWriteItems<TCaptureSample> + Send + Sync,
    TPlaybackDataReader: AsyncReadItems<TPlaybackSample> + Send + Sync,

    Backend<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>:
        audio::backend::Backend,
{
    type Backend =
        Backend<TCaptureSample, TPlaybackSample, TCaptureDataWriter, TPlaybackDataReader>;

    fn build(self) -> Result<Self::Backend, crate::Error> {
        let capture_sample_spec = format::to_pulse_sample_spec(self.continuation.capture_format);
        let playback_sample_spec = format::to_pulse_sample_spec(self.continuation.playback_format);

        let pa_record = util::build_psimple(capture_sample_spec, Direction::Record);
        let pa_playback = util::build_psimple(playback_sample_spec, Direction::Playback);

        let backend = Backend {
            capture_sample: PhantomData,
            playback_sample: PhantomData,

            capture_data_writer: self.capture_data_writer,
            playback_data_reader: self.playback_data_reader,

            pa_record,
            pa_playback,

            logger: self.continuation.logger,
        };
        Ok(backend)
    }
}
