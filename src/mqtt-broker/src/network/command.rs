use protocol::mqtt::Packet;

use super::packet::conn_ack;

pub struct Command {}

impl Command {
    pub fn new() -> Self {
        return Command {};
    }

    pub fn apply(&self) ->Packet{
        return conn_ack();
    }
}
