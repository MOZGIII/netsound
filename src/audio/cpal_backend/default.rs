pub fn input_device() -> Result<cpal::Device, super::errors::Error> {
    cpal::default_input_device().ok_or(super::errors::Error::DefaultDeviceError)
}

pub fn output_device() -> Result<cpal::Device, super::errors::Error> {
    cpal::default_output_device().ok_or(super::errors::Error::DefaultDeviceError)
}
