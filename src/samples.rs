extern crate parking_lot;

use self::parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Samples {
    vecdec: VecDeque<f32>,
}

impl Samples {
    pub fn with_capacity(capacity: usize) -> Samples {
        Samples {
            vecdec: VecDeque::with_capacity(capacity),
        }
    }

    pub fn shared_with_capacity(capacity: usize) -> SharedSamples {
        Arc::new(Mutex::new(Self::with_capacity(capacity)))
    }
}

impl std::ops::Deref for Samples {
    type Target = VecDeque<f32>;

    fn deref(&self) -> &Self::Target {
        &self.vecdec
    }
}

impl std::ops::DerefMut for Samples {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vecdec
    }
}

pub type SharedSamples = Arc<Mutex<Samples>>;
