use thiserror::Error;

#[derive(Error, Debug)]
pub enum MQTTBrokerError {
    
    #[error("Topic alias is too long. alias is {0}")]
    TopicAliasTooLong(u16),

    #[error("Topic name cannot be empty")]
    TopicNameIsEmpty,

    #[error("Topic name is incorrectly formatted")]
    TopicNameIncorrectlyFormatted,
}
