use crate::future::select_first;
use crate::log::{logger, o, LogScopeFutureExt};
use crate::transcode::Transcode;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct TranscodeService<T> {
    #[derivative(Debug = "ignore")]
    pub capture_transcoder: Box<dyn Transcode<Ok = T> + Send>,
    #[derivative(Debug = "ignore")]
    pub playback_transcoder: Box<dyn Transcode<Ok = T> + Send>,
}

impl<T> TranscodeService<T> {
    pub async fn transcode_loop(&mut self) -> Result<T, crate::Error> {
        select_first(
            self.capture_transcoder
                .transcode_loop()
                .with_logger(logger().new(o!("logger" => "transcode::capture"))),
            self.playback_transcoder
                .transcode_loop()
                .with_logger(logger().new(o!("logger" => "transcode::playback"))),
        )
        .await
    }
}
