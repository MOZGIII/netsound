//! A PCM [`Sample`] definition.

/// A PCM [`Sample`].
pub trait Sample: dasp_sample::Sample + Unpin + Send {}

impl<S> Sample for S where S: dasp_sample::Sample + Unpin + Send {}
