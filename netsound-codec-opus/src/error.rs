//! Error types.

use thiserror::Error;

/// An error that can occur during codec initialization.
#[derive(Error, Debug)]
pub enum Init {
    /// Integer conversion failed.
    #[error("try from int error: {0}")]
    TryFromInt(#[from] std::num::TryFromIntError),
    /// Underlying opus library error.
    #[error("opus error: {0}")]
    Opus(#[from] audiopus::Error),
}

/// An error that can occur during codec operation.
#[derive(Error, Debug)]
pub enum Op {
    /// The codec was invoked with less data than required.
    #[error("not enough data: {samples_available} samples available, {samples_required} samples required")]
    NotEnoughData {
        /// The effective samples amount passed to the codec.
        samples_available: usize,
        /// The required samples amount.
        samples_required: usize,
    },
    /// Underlying opus library error.
    #[error("opus error: {0}")]
    Opus(#[from] audiopus::Error),
    /// IO error while operating with the samples.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<Op> for netsound_core::codec::error::Encoding {
    fn from(err: Op) -> Self {
        match err {
            Op::NotEnoughData { .. } => {
                netsound_core::codec::error::Encoding::NotEnoughData(err.into())
            }
            err => netsound_core::codec::error::Encoding::Other(err.into()),
        }
    }
}

impl From<Op> for netsound_core::codec::error::Decoding {
    fn from(err: Op) -> Self {
        match err {
            Op::Opus(audiopus::Error::EmptyPacket) => {
                netsound_core::codec::error::Decoding::EmptyPacket(err.into())
            }
            err => netsound_core::codec::error::Decoding::Other(err.into()),
        }
    }
}
