/// The newtype used to specify the sample rate (i.e. in a context of some
/// signal defintion).
#[derive(Derivative, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[derivative(Debug = "transparent")]
#[repr(transparent)]
pub struct SampleRate(usize);

impl SampleRate {
    #[must_use]
    pub const fn from_usize(val: usize) -> Self {
        Self(val)
    }

    #[must_use]
    pub const fn as_usize(self) -> usize {
        self.0
    }
}

impl From<usize> for SampleRate {
    fn from(val: usize) -> Self {
        Self(val)
    }
}

impl From<SampleRate> for usize {
    fn from(val: SampleRate) -> Self {
        val.0
    }
}

impl std::fmt::Display for SampleRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
