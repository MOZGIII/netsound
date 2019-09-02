use crate::sync::Synced;
use crate::Error;

use super::{Backend, Format};

#[derive(Debug)]
pub struct BackendBuilder<'a, TCaptureDataWriter, TPlaybackDataReader> {
    pub capture_data_writer: Synced<TCaptureDataWriter>,
    pub playback_data_reader: Synced<TPlaybackDataReader>,

    pub request_capture_formats: &'a [Format],
    pub request_playback_formats: &'a [Format],
}

pub trait BackendBuilderFor<T: Backend>: Sized {
    fn build(self) -> Result<T, Error>;
}

pub trait BoxedBackendBuilderFor<'a, TBackend: Backend> {
    type Backend: Backend + 'a;

    fn build_boxed(self) -> Result<Box<dyn Backend + 'a>, Error>;
}

impl<'a, TBackend, TBuilder> BoxedBackendBuilderFor<'a, TBackend> for TBuilder
where
    TBackend: Backend + 'a,
    TBuilder: BackendBuilderFor<TBackend>,
{
    type Backend = TBackend;

    fn build_boxed(self) -> Result<Box<dyn Backend + 'a>, Error> {
        Ok(Box::new(self.build()?))
    }
}
