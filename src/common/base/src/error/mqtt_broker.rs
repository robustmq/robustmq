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

    #[error("Cluster is in self-protection state, please request later")]
    ClusterIsInSelfProtection,

    #[error(
        "Subscribe to push, send QOS2 message to client {0}, wait for PubRec message timeout."
    )]
    SubPublishWaitPubRecTimeout(String),

    #[error("Bad subscription Path [{0}] does not exist")]
    SubscriptionPathNotExists(String),

    #[error("User does not exist")]
    UserDoesNotExist,

    #[error("Session does not exist")]
    SessionDoesNotExist,

    #[error("Topic [{0}] does not exist")]
    TopicDoesNotExist(String),
}
