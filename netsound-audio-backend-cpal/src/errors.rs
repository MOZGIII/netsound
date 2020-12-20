use std::fmt;

#[derive(Debug)]
pub enum Error {
    DefaultDevice,
    StreamConfigNegotiation,
    SupportedFormats(cpal::SupportedFormatsError),
    BuildStream(cpal::BuildStreamError),
    DeviceName(cpal::DeviceNameError),
    PlayStream(cpal::PlayStreamError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::DefaultDevice => write!(f, "Unable to determine default audio device"),
            Error::StreamConfigNegotiation => {
                write!(f, "Unable to determine default audio stream config")
            }
            Error::SupportedFormats(err) => {
                write!(f, "Unable to fetch supported audio formats: {}", err)
            }
            Error::BuildStream(err) => write!(f, "Unable to build stream: {}", err),
            Error::DeviceName(err) => write!(f, "Unable to get device name: {}", err),
            Error::PlayStream(err) => write!(f, "Unable to play stream: {}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<cpal::SupportedFormatsError> for Error {
    fn from(err: cpal::SupportedFormatsError) -> Error {
        Error::SupportedFormats(err)
    }
}

impl From<cpal::BuildStreamError> for Error {
    fn from(err: cpal::BuildStreamError) -> Error {
        Error::BuildStream(err)
    }
}

impl From<cpal::DeviceNameError> for Error {
    fn from(err: cpal::DeviceNameError) -> Error {
        Error::DeviceName(err)
    }
}

impl From<cpal::PlayStreamError> for Error {
    fn from(err: cpal::PlayStreamError) -> Error {
        Error::PlayStream(err)
    }
}
