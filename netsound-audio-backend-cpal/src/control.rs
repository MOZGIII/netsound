use futures::channel::mpsc;

pub type Sender<T> = mpsc::Sender<T>;
pub type Receiver<T> = mpsc::Receiver<T>;

pub fn channel<T>(buffer: usize) -> (Sender<T>, Receiver<T>) {
    mpsc::channel(buffer)
}
