use super::*;
use async_trait::async_trait;

#[derive(Debug)]
pub struct Noop;

#[async_trait]
impl Transcode for Noop {
    type Ok = ();

    async fn transcode(&mut self) -> std::result::Result<Self::Ok, crate::Error> {
        Ok(())
    }
}
