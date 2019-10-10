use crate::future::select_first;
use crate::transcode::Transcode;

pub struct TranscodeService<TCaptureTranscoder, TPlaybackTranscoder> {
    pub capture_transcoder: TCaptureTranscoder,
    pub playback_transcoder: TPlaybackTranscoder,
}

impl<T, TCaptureTranscoder, TPlaybackTranscoder>
    TranscodeService<TCaptureTranscoder, TPlaybackTranscoder>
where
    TCaptureTranscoder: Transcode<Ok = T> + Send + Unpin,
    TPlaybackTranscoder: Transcode<Ok = T> + Send + Unpin,
{
    pub async fn transcode_loop(&mut self) -> Result<futures::Never, crate::Error> {
        loop {
            let _ = select_first(
                self.capture_transcoder.transcode(),
                self.playback_transcoder.transcode(),
            )
            .await?;
        }
    }
}
