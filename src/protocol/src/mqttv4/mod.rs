/*
 * Copyright (c) 2023 RobustMQ Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use super::*;
use crate::protocol::*;
use std::{str::Utf8Error, slice::Iter,fmt};
use bytes::{Buf, BufMut, Bytes, BytesMut};


pub mod connect;
pub mod connack;
pub mod publish;
pub mod puback;
pub mod pubrec;

///MQTT packet type
#[repr(u8)] 
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    Connect = 1, 
    ConnAck, 
    Publish, 
    PubAck, 
    PubRec, 
    PubRel, 
    PubComp, 
    Subscribe,
    SubAck, 
    Unsubscribe,
    UnsubAck,
    PingReq, 
    PingResp, 
    Disconnect,
}

/// Packet type from a byte
///
/// ```ignore
///          7                          3                          0
///          +--------------------------+--------------------------+
/// byte 1   | MQTT Control Packet Type | Flags for each type      |
///          +--------------------------+--------------------------+
///          |         Remaining Bytes Len  (1/2/3/4 bytes)        |
///          +-----------------------------------------------------+
///
/// <https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc385349207>
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct FixedHeader {
    /// First byte of the stream. Used to indenify packet types and several flags
    pub byte1: u8,
    /// Length of fixed header. Byte 1 + (1..4) bytes. So fixed header length can vary from 2 bytes to 5 bytes
    /// 1..4 bytes are variable length encoded to represent remaining length
    pub fixed_header_len: usize,
    /// Remaining length of the packet. Doesn't include fixed header bytes
    /// Represents variable header + payload size
    pub remaining_len: usize,
}

impl FixedHeader {
    pub fn new(byte1: u8, remaining_len_len: usize, remaining_len: usize) -> FixedHeader {
        FixedHeader {
            byte1,
            fixed_header_len: remaining_len_len + 1,
            remaining_len,
        }
    }

    pub fn packet_type(&self) -> Result<PacketType, Error> {
        let num = self.byte1 >> 4;
        match num {
            1 => Ok(PacketType::Connect),
            2 => Ok(PacketType::ConnAck),
            3 => Ok(PacketType::Publish),
            4 => Ok(PacketType::PubAck),
            5 => Ok(PacketType::PubRec),
            6 => Ok(PacketType::PubRel),
            7 => Ok(PacketType::PubComp),
            8 => Ok(PacketType::Subscribe),
            9 => Ok(PacketType::SubAck),
            10 => Ok(PacketType::Unsubscribe),
            11 => Ok(PacketType::UnsubAck), 
            12 => Ok(PacketType::PingReq),
            13 => Ok(PacketType::PingResp),
            14 => Ok(PacketType::Disconnect),
            _ => Err(Error::InvalidPacketType(num))
        }
    }

    /// Returns the size of full packet (fixed header + variable header + payload)
    /// Fixed header is enough to get the size of a frame in the stream
    pub fn frame_length(&self) -> usize {
        self.fixed_header_len + self.remaining_len
    }
}

impl fmt::Display for FixedHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Fixed_header_1st_byte:{:#010b}, Fixed_header_length:{}, Varible_header_length + Payload_length:{} ",
        self.byte1,
        self.fixed_header_len,
        self.remaining_len)
    }
}
///Parses fixed header
fn parse_fixed_header(mut stream: Iter<u8>) -> Result<FixedHeader, Error> {
    // At least 2 bytes are necesssary to frame a packet
    let stream_len = stream.len();
    if stream_len < 2 {
        return Err(Error::InsufficientBytes(2 - stream_len));
    }
    let byte1 = stream.next().unwrap();
    let (len_len, len) = length(stream)?;
    Ok(FixedHeader::new(*byte1, len_len, len))
}

/// Checks if the stream has enough bytes to frame a packet and returns fixed header
/// only if a packet can be framed with existing bytes in the `stream`.
/// The passed stream doesn't modify parent stream's cursor. If this function
/// returned an error, next `check` on the same parent stream is forced start
/// with cursor at 0 again (Iter is owned. Only Iter's cursor is changed internally)
pub fn check(stream: Iter<u8>, max_packet_size: usize) -> Result<FixedHeader, Error> {
    // Create fixed header if there are enough bytes in the stream to frame full packet
    let stream_len = stream.len();
    let fixed_header = parse_fixed_header(stream)?;

    // Don't let rogue connections attack with huge payloads.
    // Disconnect them before reading all that data
    if fixed_header.remaining_len > max_packet_size {
        return Err(Error::PayloadSizeLimitExceeded(fixed_header.remaining_len));
    }
    
    // If the current call fails due to insufficient bytes in the stream
    // after calculating remaining length, we extend the stream
    let frame_length = fixed_header.frame_length();
    if stream_len < frame_length {
        return Err(Error::InsufficientBytes(frame_length - stream_len));
    }

    Ok(fixed_header)
    
}

/// Parses variable byte integer in the stream and returns the length 
/// and number of bytes that make it. Used for remaining length calculation
/// as well as for calculating property lengths
pub fn length(stream: Iter<u8>) -> Result<(usize, usize), Error>{
    let mut len: usize = 0;
    let mut len_len = 0;
    let mut done = false;
    let mut shift = 0;
    // use continuation bit at position 7 to continue reading next byte to frame 'length'.
    // stream 0b1xxx_xxxx 0b1yyy_yyyy 0b1zzz_zzzz 0b0www_wwww will be framed as number
    // 0bwww_wwww_zzz_zzzz_yyy_yyyy_xxx_xxxxx
    for byte in stream {
        len_len += 1;
        let byte = *byte as usize;
        len += (byte & 0x7F) << shift;
        
        // stop when continue bit is 0
        done = (byte & 0x80) == 0;
        if done {
            break;
        }

        shift += 7;

        // Only a max of 4 bytes allowed for remaining length
        // more than 4 shifts (0, 7, 14, 21) implies bad length
        if shift > 21 {
            return Err(Error::MalformedRemainingLength);
        }
    }
     // Not enough bytes to frame remaining length. wait for one more byte
     if !done {
        return Err(Error::InsufficientBytes(1));
    }

    Ok((len_len, len))
}

/// Read a series of bytes with a length from a byte stream
fn read_mqtt_bytes(stream: &mut Bytes) -> Result<Bytes, Error> {
    let len = read_u16(stream)? as usize;

    // Prevent attacks with wrong remaining length. This method is used in packet.assembly() with
    // (enough) bytes to frame packet. Ensure that reading variable length string or bytes doesn't
    // cross promised boundary with 'read_fixed_header()'
    if len > stream.len() {
        return Err(Error::BoundaryCrossed(len));
    }
    Ok(stream.split_to(len))
}
/// After collecting enough bytes to frame a packet (packet's frame()), it's possible that
/// content itself in the stream is wrong. Like expected packet id or qos not being present.
/// In cases where 'read_mqtt_string' or 'read_mqtt_bytes' exhausted remaining length but
/// packet framing expects to parse qos next, these pre checks will prevent 'bytes' crashes.
pub fn read_u16(stream: &mut Bytes) -> Result<u16, Error> {
    if stream.len() < 2 {
        return Err(Error::MalformedPacket);
    }

    Ok(stream.get_u16())
}

fn read_u8(stream: &mut Bytes) -> Result<u8, Error> {
    if stream.is_empty() {
        return Err(Error::MalformedPacket);
    }
    Ok(stream.get_u8())
}
/// Serializes bytes to stream (including length)
fn write_mqtt_bytes(stream: &mut BytesMut, bytes: &[u8]) {
    stream.put_u16(bytes.len() as u16);
    stream.extend_from_slice(bytes);
}

/// Serializes a string to stream
fn write_mqtt_string(stream: &mut BytesMut, string: &str) {
    write_mqtt_bytes(stream, string.as_bytes())
}

/// Writes remaining length to stream and returns number of bytes for remaining length
pub fn write_remaining_length(stream: &mut BytesMut, len: usize) -> Result<usize, Error> {
    if len > 268_435_455 {
        return Err(Error::PayloadTooLong); // Maximum packet length is 256MB
    }

    let mut done = false;
    let mut x = len;
    let mut count = 0;

    while !done {
        let mut byte = (x % 128) as u8;
        x /= 128;
        if x > 0 {
            byte |= 128;
        }

        stream.put_u8(byte);
        count += 1;
        done = x == 0;
    }

    Ok(count)
}

#[derive(Debug, Clone)]
pub struct MqttV4;

impl MqttV4 {
    pub fn new() -> Self{
        return Self{};
    }
}

impl Protocol for MqttV4 {
    
     // Reads a stream of bytes and extracts next MQTT packet out of it
     fn read_mut(&mut self, stream: &mut BytesMut, max_size: usize) -> Result<Packet, Error> {
        let fixed_header = check(stream.iter(), max_size)?;
        // Test with a stream with exactly the size to check border panics
        let packet = stream.split_to(fixed_header.frame_length());
        let packet_type = fixed_header.packet_type()?;

        // if fixed_header.remaining_len == 0 {
        //     // no payload packets
        //     return match packet_type {

        //     }
        // }
        let packet = packet.freeze();
        let packet = match packet_type {
            PacketType::Connect => {
                let (connect, login, lastwill) = connect::read(fixed_header, packet)?;
                Packet::Connect(connect, None, lastwill, None, login)
            }
            PacketType::ConnAck => Packet::ConnAck(connack::read(fixed_header, packet)?, None),
            PacketType::Publish => Packet::Publish(publish::read(fixed_header, packet)?, None),
            _ => unreachable!(),

        };
        Ok(packet)
     }

     fn write(&self, packet: Packet, buffer: &mut BytesMut) -> Result<usize, Error> {
        let size = match packet {
            Packet::Connect(connect, None, last_will, None, login) => {
                connect::write(&connect, &login, &last_will, buffer)?
            }
            Packet::ConnAck(connack, _) => connack::write(&connack, buffer)?,
            Packet::Publish(publish, None) => publish::write(&publish, buffer)?,
            _=> unreachable!(
                "This branch only matches for packets with Properties, which is not possible in MQTT V4",
            ),
        };
        Ok(size)
     }
}

