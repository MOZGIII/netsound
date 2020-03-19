use crate::future::select_first;
use crate::log::{LogScopeFutureExt, logger, o};
use crate::transcode::Transcode;

pub struct TranscodeService<T> {
    pub capture_transcoder: Box<dyn Transcode<Ok = T> + Send>,
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
