use std::fmt;

#[derive(Debug)]
pub enum EncodingError {
    NotEnoughData,
    Other(Box<dyn std::error::Error>),
}

#[derive(Debug)]
pub enum DecodingError {
    EmptyPacket,
    Other(Box<dyn std::error::Error>),
}

impl fmt::Display for EncodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodingError::NotEnoughData => write!(f, "Not enough data to construct a packet"),
            EncodingError::Other(err) => err.fmt(f),
        }
    }
}

impl fmt::Display for DecodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodingError::EmptyPacket => write!(f, "Passed packet contained no elements"),
            DecodingError::Other(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for EncodingError {}
impl std::error::Error for DecodingError {}
