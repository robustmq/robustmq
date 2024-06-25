use thiserror::Error;

#[derive(Error, Debug)]
pub enum MQTTBrokerError {
    #[error("{0}")]
    CommmonError(String),
    
    #[error("topic alias is too long")]
    TopicAliasTo,

    #[error("topic alias is too long")]
    TopicAliasTooLong,

    #[error("Topic name cannot be empty")]
    TopicNameIsEmpty,

    #[error("Topic name is incorrectly formatted")]
    TopicNameIncorrectlyFormatted,
}
