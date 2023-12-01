use bytes::BytesMut;
use crate::{Packet, Error};

pub trait Protocol {
    fn read(&mut self, stream: &mut BytesMut, max_size: usize) -> Result<Packet, Error>;
    fn write(&self, packet: Packet, write: &mut BytesMut) -> Result<usize, Error>;
}

pub struct ProtocolMQTT4{

}

// impl Protocol for ProtocolMQTT4{
//     fn read_mut(&mut self, stream: &mut BytesMut, max_size:usize) -> Result<Packet, Erropr>{
        
//     }

//     fn write(&self, packet: Packet, buffer: &mut BytesMut) -> Result<usize,Error>{
//         Ok(3)
//     }
// }
