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


#![allow(dead_code, unused)]
pub mod mqttv4;

use std::{io, str::Utf8Error, string::FromUtf8Error};

/// This module is the place where all the protocal specifics gets abstracted
/// out and creates structures which are common across protocols. Since, MQTT
/// is the core protocol that this broker supports, a lot of structs closely
/// map to what MQTT specifies in its protocol
use bytes::{Buf, BufMut, Bytes, BytesMut};

// TODO: Handle the cases when there are no properties using Inner struct, so 
// handling of properties can be made simplier internally

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Packet {
    Connect(
        Connect,
        Option<ConnectProperties>,
        Option<LastWill>,
        Option<LastWillProperties>,
        Option<Login>,
    ),
}

/// Connection packet initialized by the client
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connect {
    /// mqtt keep alive interval
    pub keep_alive: u16,
    /// Client ID
    pub client_id: String,
    /// Clean session. Ask the broker to clear previous session
    pub clean_session: bool,
}

/// ConnectProperties can be used in MQTT Version 5
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectProperties {
    /// Expiry interval property after loosing connection
    pub session_expiry_interval: Option<u32>,
    /// Maximum simultaneous packets
    pub receive_maximum: Option<u16>,
    /// Maximum packet size
    pub max_packet_size: Option<u32>,
    /// Maximum mapping integer for a topic
    pub topic_alias_max: Option<u16>,
    pub request_response_info: Option<u8>,
    pub request_problem_info: Option<u8>,
    /// List of user properties
    pub user_properties: Vec<(String, String)>,
    /// Method of authentication 
    pub authentication_method: Option<String>,
    /// Authentication data
    pub authentication_data: Option<Bytes>,

}

/// LastWill that broker forwards on behalf of the client
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastWill {
    pub topic: Bytes, 
    pub message: Bytes, 
    pub qos: QoS, 
    pub retain: bool,
}

/// LastWillProperties can be used in MQTT Version 5
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastWillProperties {
    pub delay_interval: Option<u32>,
    pub payload_format_indicator: Option<u8>,
    pub message_expiry_interval: Option<u32>,
    pub content_type: Option<String>,
    pub response_topic: Option<String>,
    pub correlation_data: Option<Bytes>,
    pub user_properties: Vec<(String, String)>,
}

/// Quality of service
#[repr(u8)] 
#[derive(Debug, Clone, PartialEq, Eq, Default, Copy, PartialOrd)]
#[allow(clippy::enum_variant_names)]
pub enum QoS {
    #[default] 
    AtMostOnce = 0,
    AtLeastOnce = 1,
    ExactlyOnce = 2,

}

/// Maps a number to QoS
pub fn qos(num: u8) -> Option<QoS> {
    match num {
        0 => Some(QoS::AtMostOnce),
        1 => Some(QoS::AtLeastOnce),
        2 => Some(QoS::ExactlyOnce),
        qos => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Login {
    pub username: String,
    pub password: String,
}

/// Error during serialization and deserialization
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("Invalid return code received as response fro connect = {0}")]
    InvalidConnectReturnCode(u8),
    #[error("Invalid reason = {0}")]
    InvalidReason(u8),
    #[error("Invalid reason = {0}")]
    InvalidRemainingLength(usize),
    #[error("Invalid protocol used")]
    InvalidProtocol,
    #[error("Invalid protocol level {0}. Make sure right port is being used.")]
    InvalidProtocolLevel(u8),
    #[error("Invalid packet format")]
    IncorrectPacketFormat, 
    #[error("Invalid packet type = {0}")]
    InvalidPacketType(u8),
    #[error("Invalid retain forward rule = {0}")]
    InvalidRetainForwardRule(u8),
    #[error("Invalid QoS level = {0}")]
    InvalidQoS(u8),
    #[error("Payload is too long")]
    PayloadTooLong,
    #[error("Payload is required = {0}")]
    PayloadNotUtf8(#[from] Utf8Error),
    #[error("Promised boundary crossed, contains {0} bytes")]
    BoundaryCrossed(usize),
    #[error("Packet is malformed")]
    MalformedPacket,
    
}

