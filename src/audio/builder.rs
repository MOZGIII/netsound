use crate::format::Format;
use crate::io::{ReadItems, WriteItems};
use crate::sync::Synced;
use crate::Error;
use sample::Sample;

use super::Backend;

#[derive(Debug)]
pub struct BackendBuilder<
    'a,
    TCaptureSample,
    TPlaybackSample,
    TCaptureDataWriter,
    TPlaybackDataReader,
> where
    TCaptureSample: Sample,
    TPlaybackSample: Sample,
    TCaptureDataWriter: WriteItems<TCaptureSample>,
    TPlaybackDataReader: ReadItems<TPlaybackSample>,
{
    pub capture_data_writer: Synced<TCaptureDataWriter>,
    pub playback_data_reader: Synced<TPlaybackDataReader>,

    pub request_capture_formats: &'a [Format<TCaptureSample>],
    pub request_playback_formats: &'a [Format<TPlaybackSample>],
}

pub trait BackendBuilderFor<T: Backend>: Sized {
    fn build(self) -> Result<T, Error>;
}

pub trait BoxedBackendBuilderFor<'a, TBackend: Backend> {
    type Backend: Backend + 'a;

    fn build_boxed(
        self,
    ) -> Result<
        Box<
            dyn Backend<
                    CaptureSample = <TBackend as Backend>::CaptureSample,
                    PlaybackSample = <TBackend as Backend>::PlaybackSample,
                > + 'a,
        >,
        Error,
    >;
}

impl<'a, TBackend, TBuilder> BoxedBackendBuilderFor<'a, TBackend> for TBuilder
where
    TBackend: Backend + 'a,
    TBuilder: BackendBuilderFor<TBackend>,
{
    type Backend = TBackend;

    fn build_boxed(
        self,
    ) -> Result<
        Box<
            dyn Backend<
                    CaptureSample = <TBackend as Backend>::CaptureSample,
                    PlaybackSample = <TBackend as Backend>::PlaybackSample,
                > + 'a,
        >,
        Error,
    > {
        Ok(Box::new(self.build()?))
    }
}
