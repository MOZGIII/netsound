use thiserror::Error;

#[derive(Error, Debug)]
pub enum Encoding {
    #[error("not enough data to construct a packet: {0}")]
    NotEnoughData(crate::Error),
    #[error("{0}")]
    Other(crate::Error),
}

#[derive(Error, Debug)]
pub enum Decoding {
    #[error("passed packet contained no elements: {0}")]
    EmptyPacket(crate::Error),
    #[error("{0}")]
    Other(crate::Error),
}
