pub trait Sample: dasp_sample::Sample + Unpin + Send {}

impl<S> Sample for S where S: dasp_sample::Sample + Unpin + Send {}
