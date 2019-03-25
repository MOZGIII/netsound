use std::fmt;

#[derive(Debug)]
pub enum Error {
    DefaultDeviceError,
    DefaultFormatError(cpal::DefaultFormatError),
    CreationError(cpal::CreationError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::DefaultDeviceError => write!(f, "Unable to determine default audio device"),
            Error::DefaultFormatError(err) => {
                write!(f, "Unable to determine default audio format: {}", err)
            }
            Error::CreationError(err) => write!(f, "Unable to create stream: {}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<cpal::DefaultFormatError> for Error {
    fn from(err: cpal::DefaultFormatError) -> Error {
        Error::DefaultFormatError(err)
    }
}

impl From<cpal::CreationError> for Error {
    fn from(err: cpal::CreationError) -> Error {
        Error::CreationError(err)
    }
}
