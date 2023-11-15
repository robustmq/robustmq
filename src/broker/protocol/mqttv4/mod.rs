/*
 * Copyright (c) 2023 robustmq team 
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
mod connect;

/// Read a series of bytes with a length from a byte stream
fn read_mqtt_bytes(stream: &mut Bytes) -> Result<Bytes, Error> {
    let len = read_u16(stream)? as usize;

    // Prevent attacks with wrong remaining length. This method is used in packet.assembly() with 
    // (enough) bytes to frame packet. Ensure that reading variable length string or bytes doesn't
    // cross promised boundary with 'read_fixed_header()'
    if len > stream.len(){
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
    if stream.is_empty(){
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
        return Err(Error::PayloadTooLong);  // Maximum packet length is 256MB
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct FixedHeader {
    /// First byte of the stream. Used to indenify packet types and several flags
    pub byte1: u8,
    /// Length of fixed header. Byte 1 + (1..4) bytes. So fixed header length can vary from 2 bytes to 5 bytes
    /// 1..4 bytes are variable lenght encoded to represent remaining length
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
}