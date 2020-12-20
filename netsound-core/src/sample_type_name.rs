use crate::sample::Sample;
use std::intrinsics::type_name;

pub fn sample_type_name<S: Sample>() -> SampleTypeName {
    SampleTypeName(type_name::<S>())
}

pub struct SampleTypeName(&'static str);

impl std::fmt::Debug for SampleTypeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for SampleTypeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
