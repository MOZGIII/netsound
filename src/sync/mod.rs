use parking_lot::Mutex;
use std::sync::Arc;

pub type Synced<T> = Arc<Mutex<T>>;

pub fn synced<T>(t: T) -> Synced<T> {
    Arc::new(Mutex::new(t))
}
