use crate::control;

use async_trait::async_trait;
use futures::StreamExt;
use netsound_core::log::no_scopes::{crit, Logger};
use netsound_core::{audio_backend, log::no_scopes::debug};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Backend {
    pub(super) error_rx: control::Receiver<(&'static str, cpal::StreamError)>,
    pub(super) drop_tx: futures::channel::oneshot::Sender<()>,
    pub(super) logger: Logger,
}

#[async_trait]
impl audio_backend::Backend for Backend {
    async fn run(&mut self) {
        let logger = &mut self.logger;

        if let Some((origin, error)) = self.error_rx.next().await {
            crit!(
                logger,
                "an error occurred at cpal stream {:?}: {}",
                origin,
                error
            );
        } else {
            debug!(logger, "cpal backend errors stream closed");
        }
    }
}
