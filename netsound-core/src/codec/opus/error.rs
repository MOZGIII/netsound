use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("not enough data: {samples_available} samples available, {samples_required} samples required")]
    NotEnoughData {
        samples_available: usize,
        samples_required: usize,
    },
    #[error("try from int error: {0}")]
    TryFromInt(#[from] std::num::TryFromIntError),
    #[error("opus error: {0}")]
    Opus(#[from] audiopus::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<Error> for super::super::error::Encoding {
    fn from(err: Error) -> Self {
        match err {
            Error::NotEnoughData { .. } => super::super::error::Encoding::NotEnoughData(err.into()),
            err => super::super::error::Encoding::Other(err.into()),
        }
    }
}

impl From<Error> for super::super::error::Decoding {
    fn from(err: Error) -> Self {
        match err {
            Error::Opus(audiopus::Error::EmptyPacket) => {
                super::super::error::Decoding::EmptyPacket(err.into())
            }
            err => super::super::error::Decoding::Other(err.into()),
        }
    }
}
