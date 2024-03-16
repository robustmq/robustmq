use std::io;

pub mod codec;
pub mod storage;

/// Error during serialization and deserialization
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("data store disconnected")]
    IoError(#[from] io::Error),
    #[error("Payload size has been exceeded by {0} bytes")]
    PayloadSizeLimitExceeded(usize),
    #[error("Length of the request header is 0")]
    HeaderLengthIsZero,
    #[error("Error parsing request header, error message {0}")]
    DecodeHeaderError(String),
    #[error("Parsing request body error, request body identification: {0}, error message {1}")]
    DecodeBodyError(String, String),
}
