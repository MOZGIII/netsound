use cpal::traits::HostTrait;

pub fn input_device<H: HostTrait>(host: &H) -> Result<H::Device, super::errors::Error> {
    host.default_input_device()
        .ok_or(super::errors::Error::DefaultDevice)
}

pub fn output_device<H: HostTrait>(host: &H) -> Result<H::Device, super::errors::Error> {
    host.default_output_device()
        .ok_or(super::errors::Error::DefaultDevice)
}
