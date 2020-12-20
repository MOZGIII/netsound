use std::fmt;
use std::io;
use std::num;

#[derive(Debug)]
pub enum Error {
    NotEnoughData {
        samples_available: usize,
        samples_required: usize,
    },
    TryFromInt(num::TryFromIntError),
    Opus(audiopus::Error),
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotEnoughData {
                samples_available,
                samples_required,
            } => write!(
                f,
                "Not enough data: {} samples available, {} samples required",
                samples_available, samples_required
            ),
            Error::TryFromInt(err) => write!(f, "Try From Int Error: {}", err),
            Error::Opus(err) => write!(f, "Opus Error: {}", err),
            Error::Io(err) => write!(f, "IO Error: {}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<audiopus::Error> for Error {
    fn from(err: audiopus::Error) -> Self {
        Error::Opus(err)
    }
}

impl Into<super::super::EncodingError> for Error {
    fn into(self) -> super::super::EncodingError {
        match self {
            Error::NotEnoughData { .. } => super::super::EncodingError::NotEnoughData,
            err => super::super::EncodingError::Other(err.into()),
        }
    }
}

impl Into<super::super::DecodingError> for Error {
    fn into(self) -> super::super::DecodingError {
        match self {
            Error::Opus(audiopus::Error::EmptyPacket) => super::super::DecodingError::EmptyPacket,
            err => super::super::DecodingError::Other(err.into()),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<num::TryFromIntError> for Error {
    fn from(err: num::TryFromIntError) -> Self {
        Error::TryFromInt(err)
    }
}
