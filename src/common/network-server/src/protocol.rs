use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum RobustMQProtocol {
    MQTT3,
    MQTT4,
    MQTT5,
    KAFKA,
}
