use std::fmt;

#[derive(Debug)]
pub enum Error {
    DefaultDeviceError,
    FormatNegotiationError,
    FormatsEnumerationError(cpal::FormatsEnumerationError),
    CreationError(cpal::CreationError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::DefaultDeviceError => write!(f, "Unable to determine default audio device"),
            Error::FormatNegotiationError => write!(f, "Unable to determine default audio format"),
            Error::FormatsEnumerationError(err) => {
                write!(f, "Unable to determine default audio format: {}", err)
            }
            Error::CreationError(err) => write!(f, "Unable to create stream: {}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<cpal::FormatsEnumerationError> for Error {
    fn from(err: cpal::FormatsEnumerationError) -> Error {
        Error::FormatsEnumerationError(err)
    }
}

impl From<cpal::CreationError> for Error {
    fn from(err: cpal::CreationError) -> Error {
        Error::CreationError(err)
    }
}
