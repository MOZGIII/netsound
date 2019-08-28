use std::fmt;

#[derive(Debug)]
pub enum Error {
    DefaultDeviceError,
    FormatNegotiationError,
    SupportedFormatsError(cpal::SupportedFormatsError),
    BuildStreamError(cpal::BuildStreamError),
    DeviceNameError(cpal::DeviceNameError),
    PlayStreamError(cpal::PlayStreamError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::DefaultDeviceError => write!(f, "Unable to determine default audio device"),
            Error::FormatNegotiationError => write!(f, "Unable to determine default audio format"),
            Error::SupportedFormatsError(err) => {
                write!(f, "Unable to fetch supported audio formats: {}", err)
            }
            Error::BuildStreamError(err) => write!(f, "Unable to build stream: {}", err),
            Error::DeviceNameError(err) => write!(f, "Unable to get device name: {}", err),
            Error::PlayStreamError(err) => write!(f, "Unable to play stream: {}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<cpal::SupportedFormatsError> for Error {
    fn from(err: cpal::SupportedFormatsError) -> Error {
        Error::SupportedFormatsError(err)
    }
}

impl From<cpal::BuildStreamError> for Error {
    fn from(err: cpal::BuildStreamError) -> Error {
        Error::BuildStreamError(err)
    }
}

impl From<cpal::DeviceNameError> for Error {
    fn from(err: cpal::DeviceNameError) -> Error {
        Error::DeviceNameError(err)
    }
}

impl From<cpal::PlayStreamError> for Error {
    fn from(err: cpal::PlayStreamError) -> Error {
        Error::PlayStreamError(err)
    }
}
