use super::*;
use async_trait::async_trait;

#[derive(Debug)]
pub struct Noop;

#[async_trait]
impl Transcode for Noop {
    type Ok = futures::never::Never;

    async fn transcode_loop(&mut self) -> std::result::Result<Self::Ok, crate::Error> {
        loop {
            futures::pending!();
        }
    }
}
