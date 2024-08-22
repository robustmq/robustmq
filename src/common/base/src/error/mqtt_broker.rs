use std::string::FromUtf8Error;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MQTTBrokerError {
    #[error("{0}")]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error("Topic alias is too long. alias is {0}")]
    TopicAliasTooLong(u16),

    #[error("Topic name cannot be empty")]
    TopicNameIsEmpty,

    #[error("topic name is not available")]
    TopicNameInvalid(),

    #[error("Topic name is incorrectly formatted")]
    TopicNameIncorrectlyFormatted,

    #[error("Connection ID [0] information not found in cache.")]
    NotFoundConnectionInCache(u64),

    #[error("There is a problem with the length [{0}] of the Packet. Please check the length of the request packet")]
    PacketLenthError(usize),
}
