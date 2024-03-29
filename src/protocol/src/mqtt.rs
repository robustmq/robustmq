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

use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::{fmt, io, slice::Iter, str::Utf8Error, string::FromUtf8Error};

/// This module is the place where all the protocal specifics gets abstracted
/// out and creates structures which are common across protocols. Since, MQTT
/// is the core protocol that this broker supports, a lot of structs closely
/// map to what MQTT specifies in its protocol

// TODO: Handle the cases when there are no properties using Inner struct, so
// handling of properties can be made simplier internally

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MQTTPacket {
    Connect(
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
        write!(f, "Fixed_header_1st_byte:{:#010b}, Fixed_header_length:{}, Varible_header_length + Payload_length:{} ",
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
    Success,
    RefusedProtocolVersion,
    BadClientId,
    ServiceUnavailable,
    UnspecifiedError,
    MalformedPacket,
    ProtocolError,
    ImplementationSpecificError,
    UnsupportedProtocolVersion,
    ClientIdentifierNotValid,
    BadUserNamePassword,
    NotAuthorized,
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
    pub(crate) dup: bool,
    pub(crate) qos: QoS,
    pub(crate) pkid: u16,
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
    pub reason: PubAckReason,
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub reason: PubRecReason,
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub reason: PubRelReason,
}
/// Return code in pubrel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PubRelReason {
    Success,
    PacketIdentifierNotFound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub reason: PubCompReason,
}
/// Return code in pubcomp pcaket
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PubCompReason {
    Success,
    PacketIdentifierNotFound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subscribe {
    pub pkid: u16,
    pub filters: Vec<Filter>,
}

/// Subscription filter
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filter {
    //  in mqtt v4, there are only path and qos valid
    pub path: String,
    pub qos: QoS,

    // the following options are only valid in mqtt v5
    pub nolocal: bool,
    pub preserve_retain: bool,
    pub retain_forward_rule: RetainForwardRule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetainForwardRule {
    OnEverySubscribe,
    OnNewSubscribe,
    Never,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscribeProperties {
    pub id: Option<usize>,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubAckProperties {
    pub reason_string: Option<String>,
    pub user_properties: Vec<(String, String)>,
}

//--------------------------- Unsubscribe packet -------------------------------

/// Unsubscribe packet
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unsubscribe {
    pub pkid: u16,
    pub filters: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsubAckProperties {
    pub reason_string: Option<String>,
    pub user_properties: Vec<(String, String)>,
}

//--------------------------- Disconnect packet -------------------------------
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Disconnect {
    /// Disconnect reason code which will be only used in MQTT V5
    pub reason_code: DisconnectReasonCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DisconnectReasonCode {
    /// Close the connection normally, don't send the will message.
    NormalDisconnection,
    /// The client wishes to disconnect but requires that the server also publishes its
    /// Will message.
    DisconnectWithWillMessage,
    /// The Connection is closed but the sender either does not wish to reveal the reason,
    /// or none of the other reason codes apply.
    UnspecifiedError,
    /// The received packet does not conform to this specification.
    MalformedPacket,
    /// An unexpected or out of order packet was received.
    ProtocolError,
    /// The packet received is valid but cannot be processed by this implementation.
    ImplementationSpecificError,
    /// The request is not authorized.
    NotAuthorized,
    /// The Server is busy and cannot continue processing requests from this Client.
    ServerBusy,
    /// The Server is shutting down.
    ServerShuttingDown,
    /// The Connection is closed because no packet has been received for 1.5 times the Keepalive time.
    KeepAliveTimeout,
    /// Another Connection using the same ClientID has connected causing this Connection to be closed.
    SessionTakenOver,
    /// The Topic Filter is correctly formed, but is not accepted by this Sever.
    TopicFilterInvalid,
    /// The Topic Name is correctly formed, but is not accepted by this Client or Server.
    TopicNameInvalid,
    /// The Client or Server has received more than Receive Maximum publication for which it has not sent PUBACK or PUBCOMP.
    ReceiveMaximumExceeded,
    /// The Client or Server has received a PUBLISH packet containing a Topic Alias which is greater than the Maximum Topic Alias it sent in the CONNECT or CONNACK packet.
    TopicAliasInvalid,
    /// The packet size is greater than Maximum Packet Size for this Client or Server.
    PacketTooLarge,
    /// The received data rate is too high.
    MessageRateTooHigh,
    /// An implementation or administrative imposed limit has been exceeded.
    QuotaExceeded,
    /// The Connection is closed due to an administrative action.
    AdministrativeAction,
    /// The payload format does not match the one specified by the Payload Format Indicator.
    PayloadFormatInvalid,
    /// The Server has does not support retained messages.
    RetainNotSupported,
    /// The Client specified a QoS greater than the QoS specified in a Maximum QoS in the CONNACK.
    QoSNotSupported,
    /// The Client should temporarily change its Server.
    UseAnotherServer,
    /// The Server is moved and the Client should permanently change its server location.
    ServerMoved,
    /// The Server does not support Shared Subscriptions.
    SharedSubscriptionNotSupported,
    /// This connection is closed because the connection rate is too high.
    ConnectionRateExceeded,
    /// The maximum connection time authorized for this connection has been exceeded.
    MaximumConnectTime,
    /// The Server does not support Subscription Identifiers; the subscription is not accepted.
    SubscriptionIdentifiersNotSupported,
    /// The Server does not support Wildcard subscription; the subscription is not accepted.
    WildcardSubscriptionsNotSupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    #[error("Packet recieved has id Zero")]
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
    #[error("data store disconnected")]
    IoError(#[from] io::Error),
}

pub trait Protocol {
    fn read_mut(&mut self, stream: &mut BytesMut, max_size: usize) -> Result<MQTTPacket, Error>;
    fn write(&self, packet: MQTTPacket, write: &mut BytesMut) -> Result<usize, Error>;
}
