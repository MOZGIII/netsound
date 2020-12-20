/// Backend's only responsibility is to manage the runtime.
pub trait Backend: Send + Sync {
    fn run(&mut self);
}
