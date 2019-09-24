pub trait Sample: sample::Sample + Unpin + Send {}

impl<S> Sample for S where S: sample::Sample + Unpin + Send {}
