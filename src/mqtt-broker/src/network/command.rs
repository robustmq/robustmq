use protocol::mqtt::MQTTPacket;

use super::packet::conn_ack;

pub struct Command {}

impl Command {
    pub fn new() -> Self {
        return Command {};
    }

    pub fn apply(&self) ->MQTTPacket{
        return conn_ack();
    }
}
