// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::slice::Iter;
use std::str::Utf8Error;
use std::{fmt, io};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, PartialEq, Debug, Serialize, Deserialize)]
pub enum MqttProtocol {
    #[default]
    Mqtt3,
    Mqtt4,
    Mqtt5,
}

pub fn is_mqtt3(protocol: u8) -> bool {
    protocol == 3
}

pub fn is_mqtt4(protocol: u8) -> bool {
    protocol == 4
}

pub fn is_mqtt5(protocol: u8) -> bool {
    protocol == 5
}

impl MqttProtocol {
    pub fn is_mqtt3(&self) -> bool {
        MqttProtocol::Mqtt3.eq(self)
    }
    pub fn is_mqtt4(&self) -> bool {
        MqttProtocol::Mqtt4.eq(self)
    }
    pub fn is_mqtt5(&self) -> bool {
        MqttProtocol::Mqtt5.eq(self)
    }
}

impl From<MqttProtocol> for String {
    fn from(protocol: MqttProtocol) -> Self {
        match protocol {
            MqttProtocol::Mqtt3 => "MQTT3".into(),
            MqttProtocol::Mqtt4 => "MQTT4".into(),
            MqttProtocol::Mqtt5 => "MQTT5".into(),
        }
    }
}

impl From<MqttProtocol> for u8 {
    fn from(protocol: MqttProtocol) -> Self {
        match protocol {
            MqttProtocol::Mqtt3 => 3,
            MqttProtocol::Mqtt4 => 4,
            MqttProtocol::Mqtt5 => 5,
        }
    }
}

/// MQTT packet type
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
    Auth,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MqttPacket {
    Connect(
        u8,
        Connect,
        Option<ConnectProperties>,
        Option<LastWill>,
        Option<LastWillProperties>,
        Option<Login>,
    ),
    ConnAck(ConnAck, Option<ConnAckProperties>),
    Publish(Publish, Option<PublishProperties>),
    PubAck(PubAck, Option<PubAckProperties>),
    PubRec(PubRec, Option<PubRecProperties>),
    PubRel(PubRel, Option<PubRelProperties>),
    PubComp(PubComp, Option<PubCompProperties>),
    PingReq(PingReq),
    PingResp(PingResp),
    Subscribe(Subscribe, Option<SubscribeProperties>),
    SubAck(SubAck, Option<SubAckProperties>),
    Unsubscribe(Unsubscribe, Option<UnsubscribeProperties>),
    UnsubAck(UnsubAck, Option<UnsubAckProperties>),
    Disconnect(Disconnect, Option<DisconnectProperties>),
    Auth(Auth, Option<AuthProperties>),
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
/// <https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901134>
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
            _ => Err(Error::InvalidPacketType(num)),
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
        write!(f, "Fixed_header_1st_byte:{:#010b}, Fixed_header_length:{}, Variable_header_length + Payload_length:{} ",
        self.byte1,
        self.fixed_header_len,
        self.remaining_len)
    }
}

/// Parses fixed header
pub fn parse_fixed_header(mut stream: Iter<u8>) -> Result<FixedHeader, Error> {
    // At least 2 bytes are necessary to frame a packet
    let stream_len = stream.len();
    if stream_len < 2 {
        return Err(Error::InsufficientBytes(2 - stream_len));
    }
    let byte1 = stream.next().unwrap();
    let (len_len, len) = length(stream)?;
    Ok(FixedHeader::new(*byte1, len_len, len))
}

/// Parses variable byte integer in the stream and returns the length
/// and number of bytes that make it. Used for remaining length calculation
/// as well as for calculating property lengths
pub fn length(stream: Iter<u8>) -> Result<(usize, usize), Error> {
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

/// Read a series of bytes with a length from a byte stream
pub fn read_mqtt_bytes(stream: &mut Bytes) -> Result<Bytes, Error> {
    let len = read_u16(stream)? as usize;

    // Prevent attacks with wrong remaining length. This method is used in packet.assembly() with
    // (enough) bytes to frame packet. Ensure that reading variable length string or bytes doesn't
    // cross promised boundary with 'read_fixed_header()'
    if len > stream.len() {
        return Err(Error::BoundaryCrossed(len));
    }
    Ok(stream.split_to(len))
}

// Reads a string from bytes stream
pub fn read_mqtt_string(stream: &mut Bytes) -> Result<String, Error> {
    let s = read_mqtt_bytes(stream)?;
    match String::from_utf8(s.to_vec()) {
        Ok(v) => Ok(v),
        Err(_e) => Err(Error::TopicNotUtf8),
    }
}

pub fn read_u16(stream: &mut Bytes) -> Result<u16, Error> {
    if stream.len() < 2 {
        return Err(Error::MalformedPacket);
    }

    Ok(stream.get_u16())
}

pub fn read_u8(stream: &mut Bytes) -> Result<u8, Error> {
    if stream.is_empty() {
        return Err(Error::MalformedPacket);
    }
    Ok(stream.get_u8())
}

pub fn read_u32(stream: &mut Bytes) -> Result<u32, Error> {
    if stream.len() < 4 {
        return Err(Error::MalformedPacket);
    }

    Ok(stream.get_u32())
}

/// Serializes bytes to stream (including length)
pub fn write_mqtt_bytes(stream: &mut BytesMut, bytes: &[u8]) {
    stream.put_u16(bytes.len() as u16);
    stream.extend_from_slice(bytes);
}

/// Serializes a string to stream
pub fn write_mqtt_string(stream: &mut BytesMut, string: &str) {
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

/// Return number of remaining length bytes required for encoding length
pub fn len_len(len: usize) -> usize {
    if len >= 2_097_152 {
        4
    } else if len >= 16_384 {
        3
    } else if len >= 128 {
        2
    } else {
        1
    }
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
#[derive(Debug, Clone, PartialEq, Eq, Default)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LastWill {
    pub topic: Bytes,
    pub message: Bytes,
    pub qos: QoS,
    pub retain: bool,
}

/// LastWillProperties can be used in MQTT Version 5
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
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
#[derive(Debug, Clone, PartialEq, Eq, Default, Copy, PartialOrd, Serialize, Deserialize)]
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
        _ => None,
    }
}

impl From<QoS> for u8 {
    fn from(value: QoS) -> Self {
        match value {
            QoS::AtMostOnce => 0,
            QoS::AtLeastOnce => 1,
            QoS::ExactlyOnce => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Login {
    pub username: String,
    pub password: String,
}

impl fmt::Display for Connect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "client_id:{:?}, clean_session:{}, keep_alive:{}s ",
            self.client_id, self.clean_session, self.keep_alive
        )
    }
}

impl fmt::Display for LastWill {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "will_topic:{:?}, qos:{:?}, will_message:{:?}, retain:{} ",
            self.topic, self.qos, self.message, self.retain
        )
    }
}

impl fmt::Display for Login {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "username:{:?}, password:{:?} ",
            self.username, self.password
        )
    }
}

// ConnectionProperties only valid in MQTT V5
impl fmt::Display for ConnectProperties {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "session_expiry_interval:{:?}, receive_maximum:{:?}, max_packet_size:{:?}, topic_alias_max:{:?},
        request_response_info:{:?}, request_problem_info:{:?}, user_properties:{:?}, authentication_method:{:?},
        authentication_data:{:?}",
        self.session_expiry_interval,
        self.receive_maximum,
        self.max_packet_size,
        self.topic_alias_max,
        self.request_response_info,
        self.request_problem_info,
        self.user_properties,
        self.authentication_method,
        self.authentication_data)
    }
}

// LastWillProperties only valid in MQTT V5
impl fmt::Display for LastWillProperties {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "delay_interval:{:?}, payload_format_indicator:{:?}, message_expiry_interval:{:?}, content_type:{:?},
        response_topic:{:?}, correlation_data:{:?}, user_properties:{:?}",
        self.delay_interval,
        self.payload_format_indicator,
        self.message_expiry_interval,
        self.content_type,
        self.response_topic,
        self.correlation_data,
        self.user_properties)
    }
}

//-----------------------------ConnectAck packet----------------
/// Return code in connack
/// This contains return codes for both MQTT v4(3.11) and v5
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectReturnCode {
    // MQTT 3/4/5 Common
    Success,
    NotAuthorized,

    // MQTT 5
    UnspecifiedError,
    MalformedPacket,
    ProtocolError,
    ImplementationSpecificError,
    UnsupportedProtocolVersion,
    ClientIdentifierNotValid,
    BadUserNamePassword,
    ServerUnavailable,
    ServerBusy,
    Banned,
    BadAuthenticationMethod,
    TopicNameInvalid,
    PacketTooLarge,
    QuotaExceeded,
    PayloadFormatInvalid,
    RetainNotSupported,
    QoSNotSupported,
    UseAnotherServer,
    ServerMoved,
    ConnectionRateExceeded,

    // MQTT 3/4
    RefusedProtocolVersion,
    BadClientId,
    ServiceUnavailable,
}

/// Acknowledgement to connect packet
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnAck {
    pub session_present: bool,
    pub code: ConnectReturnCode,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConnAckProperties {
    pub session_expiry_interval: Option<u32>,
    pub receive_max: Option<u16>,
    pub max_qos: Option<u8>,
    pub retain_available: Option<u8>,
    pub max_packet_size: Option<u32>,
    pub assigned_client_identifier: Option<String>,
    pub topic_alias_max: Option<u16>,
    pub reason_string: Option<String>,
    pub user_properties: Vec<(String, String)>,
    pub wildcard_subscription_available: Option<u8>,
    pub subscription_identifiers_available: Option<u8>,
    pub shared_subscription_available: Option<u8>,
    pub server_keep_alive: Option<u16>,
    pub response_information: Option<String>,
    pub server_reference: Option<String>,
    pub authentication_method: Option<String>,
    pub authentication_data: Option<Bytes>,
}

impl fmt::Display for ConnAck {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "session_present:{}, return_code:{:?} ",
            self.session_present, self.code
        )
    }
}

impl fmt::Display for ConnAckProperties {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "session_expiry_interval:{:?}, receive_max:{:?}, max_qos:{:?}, retain_available:{:?}, max_packet_size:{:?}
        assigned_client_identifier:{:?}, topic_alias_max:{:?}, reason_string:{:?}, user_properties:{:?},
        wildcard_subscription_available:{:?}, subscription_identifiers_available:{:?},
        shared_subscription_available:{:?}, server_keep_alive:{:?}, response_information:{:?}
        server_reference:{:?}, authentication_method:{:?}, authentication_data:{:?}",

        self.session_expiry_interval,
        self.receive_max,
        self.max_qos,
        self.retain_available,
        self.max_packet_size,
        self.assigned_client_identifier,
        self.topic_alias_max,
        self.reason_string,
        self.user_properties,
        self.wildcard_subscription_available,
        self.subscription_identifiers_available,
        self.shared_subscription_available,
        self.server_keep_alive,
        self.response_information,
        self.server_reference,
        self.authentication_method,
        self.authentication_data)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PingReq;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PingResp;
//----------------------Publish Packet --------------------------------

/// Publish packet
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Publish {
    pub dup: bool,
    pub qos: QoS,
    pub pkid: u16,
    pub retain: bool,
    pub topic: Bytes,
    pub payload: Bytes,
}

impl Publish {
    //Constructor of Publish
    pub fn new<T: Into<Bytes>>(topic: T, payload: T, retain: bool) -> Publish {
        Publish {
            dup: false,
            qos: QoS::AtMostOnce,
            pkid: 0,
            retain,
            topic: topic.into(),
            payload: payload.into(),
        }
    }

    pub fn is_empty(&self) -> bool {
        false
    }

    pub fn len(&self) -> usize {
        let len = 2 + self.topic.len() + self.payload.len();
        match self.qos == QoS::AtMostOnce {
            true => len,
            false => len + 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PublishProperties {
    pub payload_format_indicator: Option<u8>,
    pub message_expiry_interval: Option<u32>,
    pub topic_alias: Option<u16>,
    pub response_topic: Option<String>,
    pub correlation_data: Option<Bytes>,
    pub user_properties: Vec<(String, String)>,
    pub subscription_identifiers: Vec<usize>,
    pub content_type: Option<String>,
}

impl fmt::Display for Publish {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "topic:{:?}, payload:{:?}, dup:{}, qos:{:?}, message_identifier:{}, retain:{} ",
            self.topic, self.payload, self.dup, self.qos, self.pkid, self.retain
        )
    }
}

impl fmt::Display for PublishProperties {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "payload_format_indicator:{:?}, message_expiry_interval:{:?}, topic_alias:{:?},
        response_topic:{:?}, correlation_data:{:?}, user_properties:{:?}, subscription_identifiers:{:?},
        content_type:{:?},",
        self.payload_format_indicator,
        self.message_expiry_interval,
        self.topic_alias,
        self.response_topic,
        self.correlation_data,
        self.user_properties,
        self.subscription_identifiers,
        self.content_type)
    }
}

/// Acknowledgement to QoS 1 publish packet
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubAck {
    pub pkid: u16,
    pub reason: Option<PubAckReason>,
}

/// Return code in puback
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PubAckReason {
    Success,
    NoMatchingSubscribers,
    UnspecifiedError,
    ImplementationSpecificError,
    NotAuthorized,
    TopicNameInvalid,
    PacketIdentifierInUse,
    QuotaExceeded,
    PayloadFormatInvalid,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PubAckProperties {
    pub reason_string: Option<String>,
    pub user_properties: Vec<(String, String)>,
}

impl fmt::Display for PubAck {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "pkid:{:?}, reason:{:?}", self.pkid, self.reason)
    }
}

impl fmt::Display for PubAckProperties {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "reason_string:{:?}, user_properties:{:?}",
            self.reason_string, self.user_properties
        )
    }
}
//-----------------------------PubRec packet---------------------------------
/// Acknowledgement as part 1 to QoS 2 publish packet
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubRec {
    pub pkid: u16,
    pub reason: Option<PubRecReason>,
}

/// Return code in pubrec packet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PubRecReason {
    Success,
    NoMatchingSubscribers,
    UnspecifiedError,
    ImplementationSpecificError,
    NotAuthorized,
    TopicNameInvalid,
    PacketIdentifierInUse,
    QuotaExceeded,
    PayloadFormatInvalid,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PubRecProperties {
    pub reason_string: Option<String>,
    pub user_properties: Vec<(String, String)>,
}

impl fmt::Display for PubRec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "pkid:{:?}, reason:{:?}", self.pkid, self.reason)
    }
}

impl fmt::Display for PubRecProperties {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "reason_string:{:?}, user_properties:{:?}",
            self.reason_string, self.user_properties
        )
    }
}

//--------------------------- PubRel packet -------------------------------

/// Publish release in response to PubRec packet as QoS 2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubRel {
    pub pkid: u16,
    pub reason: Option<PubRelReason>,
}
/// Return code in pubrel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PubRelReason {
    Success,
    PacketIdentifierNotFound,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PubRelProperties {
    pub reason_string: Option<String>,
    pub user_properties: Vec<(String, String)>,
}

impl fmt::Display for PubRel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "publish_identifier:{}, return_code:{:?}",
            self.pkid, self.reason
        )
    }
}

impl fmt::Display for PubRelProperties {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "reason_string:{:?}, user_properties:{:?}",
            self.reason_string, self.user_properties
        )
    }
}

//--------------------------- PubComp packet -------------------------------

/// Asssured publish complete as QoS 2 in response to PubRel packet
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubComp {
    pub pkid: u16,
    pub reason: Option<PubCompReason>,
}
/// Return code in pubcomp pcaket
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PubCompReason {
    Success,
    PacketIdentifierNotFound,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PubCompProperties {
    pub reason_string: Option<String>,
    pub user_properties: Vec<(String, String)>,
}

impl fmt::Display for PubComp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "publish_identifier:{}, return_code:{:?}",
            self.pkid, self.reason
        )
    }
}

impl fmt::Display for PubCompProperties {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "reason_string:{:?}, user_properties:{:?}",
            self.reason_string, self.user_properties
        )
    }
}

//--------------------------- Subscribe packet -------------------------------

/// Subscription packet
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subscribe {
    pub packet_identifier: u16,
    pub filters: Vec<Filter>,
}

/// Subscription filter
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Filter {
    //  in mqtt v4, there are only path and qos valid
    pub path: String,
    pub qos: QoS,

    // the following options are only valid in mqtt v5
    pub nolocal: bool,
    pub preserve_retain: bool,
    pub retain_forward_rule: RetainForwardRule,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetainForwardRule {
    #[default]
    OnEverySubscribe,
    OnNewSubscribe,
    Never,
}

impl From<RetainForwardRule> for u8 {
    fn from(value: RetainForwardRule) -> Self {
        match value {
            RetainForwardRule::OnEverySubscribe => 0,
            RetainForwardRule::OnNewSubscribe => 1,
            RetainForwardRule::Never => 2,
        }
    }
}

pub fn retain_forward_rule(num: u8) -> Option<RetainForwardRule> {
    match num {
        0 => Some(RetainForwardRule::OnEverySubscribe),
        1 => Some(RetainForwardRule::OnNewSubscribe),
        2 => Some(RetainForwardRule::Never),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SubscribeProperties {
    pub subscription_identifier: Option<usize>,
    pub user_properties: Vec<(String, String)>,
}

//--------------------------- SubscribeAck packet -------------------------------
/// Ackknowledgement to subscribe
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubAck {
    pub pkid: u16,
    pub return_codes: Vec<SubscribeReasonCode>,
}

impl SubAck {
    pub fn is_empty(&self) -> bool {
        false
    }

    pub fn len(&self) -> usize {
        2 + self.return_codes.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubscribeReasonCode {
    QoS0,
    QoS1,
    QoS2,
    Success(QoS),
    Failure,
    // the following codes only valid in mqtt v5
    Unspecified,
    ImplementationSpecific,
    NotAuthorized,
    TopicFilterInvalid,
    PkidInUse,
    QuotaExceeded,
    SharedSubscriptionsNotSupported,
    SubscriptionIdNotSupported,
    WildcardSubscriptionsNotSupported,
    ExclusiveSubscriptionDisabled,
    TopicSubscribed,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SubAckProperties {
    pub reason_string: Option<String>,
    pub user_properties: Vec<(String, String)>,
}

//--------------------------- Unsubscribe packet -------------------------------

/// Unsubscribe packet
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Unsubscribe {
    pub pkid: u16,
    pub filters: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UnsubscribeProperties {
    pub user_properties: Vec<(String, String)>,
}

//--------------------------- UnsubscribeAck packet -------------------------------

/// Acknowledgement to unsubscribe
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsubAck {
    pub pkid: u16,
    pub reasons: Vec<UnsubAckReason>, // only valid in MQTT V5
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum UnsubAckReason {
    Success,
    NoSubscriptionExisted,
    UnspecifiedError,
    ImplementationSpecificError,
    NotAuthorized,
    TopicFilterInvalid,
    PacketIdentifierInUse,
}

// UnsubAckProperties only valid in MQTT V5
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UnsubAckProperties {
    pub reason_string: Option<String>,
    pub user_properties: Vec<(String, String)>,
}

//--------------------------- Disconnect packet -------------------------------
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Disconnect {
    /// Disconnect reason code which will be only used in MQTT V5
    pub reason_code: Option<DisconnectReasonCode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DisconnectReasonCode {
    NormalDisconnection,
    DisconnectWithWillMessage,
    UnspecifiedError,
    MalformedPacket,
    ProtocolError,
    ImplementationSpecificError,
    NotAuthorized,
    ServerBusy,
    ServerShuttingDown,
    KeepAliveTimeout,
    SessionTakenOver,
    TopicFilterInvalid,
    TopicNameInvalid,
    ReceiveMaximumExceeded,
    TopicAliasInvalid,
    PacketTooLarge,
    MessageRateTooHigh,
    QuotaExceeded,
    AdministrativeAction,
    PayloadFormatInvalid,
    RetainNotSupported,
    QoSNotSupported,
    UseAnotherServer,
    ServerMoved,
    SharedSubscriptionNotSupported,
    ConnectionRateExceeded,
    MaximumConnectTime,
    SubscriptionIdentifiersNotSupported,
    WildcardSubscriptionsNotSupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DisconnectProperties {
    /// Session Expiry Interval in seconds
    pub session_expiry_interval: Option<u32>,

    /// Human readable reason for the disconnect
    pub reason_string: Option<String>,

    /// List of user properties
    pub user_properties: Vec<(String, String)>,

    /// String which can be used by the client to identify aonther server to use
    pub server_reference: Option<String>,
}

//--------------------------- AUTH packet -------------------------------

/// AUTH packet is for authentication, but only used in MQTT 5.0
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Auth {
    pub reason: Option<AuthReason>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AuthReason {
    Success,
    ContinueAuthentication,
    ReAuthenticate,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AuthProperties {
    /// authentication method
    pub authentication_method: Option<String>,

    /// authentication data
    pub authentication_data: Option<Bytes>,

    /// Human readable reason for the disconnect
    pub reason_string: Option<String>,

    /// List of user properties
    pub user_properties: Vec<(String, String)>,
}

impl fmt::Display for Auth {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "reason_code:{:?}", self.reason)
    }
}

impl fmt::Display for AuthProperties {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "authentication_method:{:?}, authentication_data:{:?},reason_string:{:?}, user_properties:{:?}",
            self.authentication_method, self.authentication_data, self.reason_string, self.user_properties
        )
    }
}

/// Error during serialization and deserialization
#[derive(Debug, thiserror::Error)]
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
    #[error("Remaining length is malformed")]
    MalformedRemainingLength,
    #[error("Insufficient number of bytes to frame packet, {0} more bytes required")]
    InsufficientBytes(usize),
    #[error("Packet received has id Zero")]
    PacketIdZero,
    #[error("Payload size has been exceeded by {0} bytes")]
    PayloadSizeLimitExceeded(usize),
    #[error("Empty Subscription")]
    EmptySubscription,
    #[error("Invalid subscribe reason code = {0}")]
    InvalidSubscribeReasonCode(u8),
    #[error("Topic not utf-8")]
    TopicNotUtf8,
    #[error("Payload size is incorrect")]
    PayloadSizeIncorrect,
    #[error("Invalid property type = {0}")]
    InvalidPropertyType(u8),

    #[error(transparent)]
    IoError(#[from] io::Error),
}

pub trait Protocol {
    fn read_mut(&mut self, stream: &mut BytesMut, max_size: usize) -> Result<MqttPacket, Error>;
    fn write(&self, packet: MqttPacket, write: &mut BytesMut) -> Result<usize, Error>;
}

// This type is used to avoid #[warn(clippy::type_complexity)]
pub struct ConnectReadOutcome {
    pub protocol_version: u8,
    pub connect: Connect,
    pub properties: Option<ConnectProperties>,
    pub last_will: Option<LastWill>,
    pub last_will_properties: Option<LastWillProperties>,
    pub login: Option<Login>,
}

pub fn connect_read(
    fixed_header: FixedHeader,
    mut bytes: Bytes,
) -> Result<ConnectReadOutcome, Error> {
    let variable_header_index = fixed_header.fixed_header_len;
    bytes.advance(variable_header_index);

    // variable header
    let protocol_name = read_mqtt_string(&mut bytes)?;
    let protocol_level = read_u8(&mut bytes)?;
    if protocol_name != "MQTT" && protocol_name != "MQIsdp" {
        return Err(Error::InvalidProtocol);
    }

    if protocol_level == 5 {
        let connect_flags = read_u8(&mut bytes)?;
        let clean_session = (connect_flags & 0b10) != 0;
        let keep_alive = read_u16(&mut bytes)?;

        let properties = crate::mqtt::mqttv5::connect::properties::read(&mut bytes)?;
        let client_id = read_mqtt_string(&mut bytes)?;
        let (will, willproperties) =
            crate::mqtt::mqttv5::connect::will::read(connect_flags, &mut bytes)?;
        let login = crate::mqtt::mqttv5::connect::login::read(connect_flags, &mut bytes)?;

        let connect = Connect {
            keep_alive,
            client_id,
            clean_session,
        };

        return Ok(ConnectReadOutcome {
            protocol_version: protocol_level,
            connect,
            properties,
            last_will: will,
            last_will_properties: willproperties,
            login,
        });
    }

    if protocol_level == 4 || protocol_level == 3 {
        let connect_flags = read_u8(&mut bytes)?;
        let clean_session = (connect_flags & 0b10) != 0;
        let keep_alive = read_u16(&mut bytes)?;
        let client_id = read_mqtt_bytes(&mut bytes)?;
        let client_id = std::str::from_utf8(&client_id)?.to_owned();
        let last_will = crate::mqtt::mqttv4::connect::will::read(connect_flags, &mut bytes)?;
        let login = crate::mqtt::mqttv4::connect::login::read(connect_flags, &mut bytes)?;

        let connect = Connect {
            keep_alive,
            client_id,
            clean_session,
        };

        return Ok(ConnectReadOutcome {
            protocol_version: protocol_level,
            connect,
            properties: None,
            last_will,
            last_will_properties: None,
            login,
        });
    }

    Err(Error::InvalidProtocol)
}
