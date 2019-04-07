use std::fmt;

#[derive(Debug)]
pub enum EncodingError {
    Other(Box<std::error::Error>),
}

#[derive(Debug)]
pub enum DecodingError {
    EmptyPacket,
    Other(Box<std::error::Error>),
}

impl fmt::Display for EncodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EncodingError::Other(err) => err.fmt(f),
        }
    }
}

impl fmt::Display for DecodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DecodingError::EmptyPacket => write!(f, "Passed packet contained no elements"),
            DecodingError::Other(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for EncodingError {}
impl std::error::Error for DecodingError {}
