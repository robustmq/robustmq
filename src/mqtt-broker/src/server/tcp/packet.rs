#[derive(Clone, PartialEq)]
pub enum Protocol {
    Mqtt4,
    Mqtt5,
}

#[derive(Debug)]
pub struct RequestPackage<T> {
    pub connection_id: u64,
    pub packet: T,
}

impl<T> RequestPackage<T> {
    pub fn new(connection_id: u64, packet: T) -> Self {
        Self {
            connection_id,
            packet,
        }
    }
}

#[derive(Debug)]
pub struct ResponsePackage<T> {
    pub connection_id: u64,
    pub packet: T,
}

impl<T> ResponsePackage<T> {
    pub fn new(connection_id: u64, packet: T) -> Self {
        Self {
            connection_id,
            packet,
        }
    }
}
