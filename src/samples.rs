use parking_lot;

use self::parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Samples {
    vecdec: VecDeque<f32>,
}

pub type SharedSamples = Arc<Mutex<Samples>>;

impl Samples {
    pub fn with_capacity(capacity: usize) -> Samples {
        Samples {
            vecdec: VecDeque::with_capacity(capacity),
        }
    }

    pub fn shared_with_capacity(capacity: usize) -> SharedSamples {
        Arc::new(Mutex::new(Self::with_capacity(capacity)))
    }

    pub fn read_f32(&mut self, buf: &mut [f32]) -> usize {
        let vecdec = &mut self.vecdec;
        let mut filled: usize = 0;
        for chunk in buf.iter_mut() {
            match vecdec.pop_front() {
                None => break,
                Some(sample) => {
                    *chunk = sample;
                    filled += 1;
                }
            }
        }
        filled
    }

    pub fn write_f32(&mut self, buf: &[f32]) -> usize {
        let vecdec = &mut self.vecdec;
        let mut filled: usize = 0;
        for chunk in buf.iter() {
            vecdec.push_back(*chunk);
            filled += 1;
        }
        filled
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
