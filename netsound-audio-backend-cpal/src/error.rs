use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unable to determine default audio device")]
    DefaultDevice,
    #[error("unable to negotiate audio stream config")]
    StreamConfigNegotiation,
    #[error("unable to fetch supported audio stream configs: {0}")]
    SupportedStreamConfigs(#[from] cpal::SupportedStreamConfigsError),
    #[error("unable to build stream: {0}")]
    BuildStream(#[from] cpal::BuildStreamError),
    #[error("unable to get device name: {0}")]
    DeviceName(#[from] cpal::DeviceNameError),
    #[error("unable to play stream: {0}")]
    PlayStream(#[from] cpal::PlayStreamError),
}
