#![allow(non_camel_case_types, unused)]

use std::collections::HashSet;
use std::mem::size_of;
use std::str::from_utf8;
use serde::de::Unexpected::Bool;
use crate::config::LogLevel;
use crate::types::{ClientId, ObjectId, PingId, Size};

/** ==================================
*
*      ENUMERATIONS STRUCT &
*         GLOBAL CLASSES
*
** ================================*/

/**
 * MessageType are used to translate request type
 * to the corresponding hexadecimal code.
 */
#[derive(Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum MessageType {
    CONNECT,
    CONNECT_ACK,
    OPEN_STREAM,
    SHUTDOWN,
    HEARTBEAT,
    HEARTBEAT_REQUEST,
    PING,
    PONG,
    TOPIC_REQUEST,
    TOPIC_REQUEST_ACK,
    TOPIC_REQUEST_NACK,
    OBJECT_REQUEST,
    OBJECT_REQUEST_ACK,
    OBJECT_REQUEST_NACK,
    DATA,
    SERVER_STATUS,
    SERVER_STATUS_ACK,
    UNKNOWN,
}

/**
 * This function return the string name of the MessageType given.
 *
 * @param message: MessageType, the source to translate into string.
 *
 * @return string, the corresponding name
 */
pub fn display_message_type<'a>(message: MessageType) -> &'a str {
    match message {
        MessageType::CONNECT => "Connect",
        MessageType::CONNECT_ACK => "Connect_ACK",
        MessageType::OPEN_STREAM => "Open_Stream",
        MessageType::SHUTDOWN => "Shutdown",
        MessageType::HEARTBEAT => "HeartBeat",
        MessageType::HEARTBEAT_REQUEST => "HeartBeat_Request",
        MessageType::PING => "Ping",
        MessageType::PONG => "Pong",
        MessageType::TOPIC_REQUEST => "Topic_Request",
        MessageType::TOPIC_REQUEST_ACK => "Topic_Request_Ack",
        MessageType::TOPIC_REQUEST_NACK => "Topic_Request_Nack",
        MessageType::OBJECT_REQUEST => "Object_Request",
        MessageType::OBJECT_REQUEST_ACK => "Object_Request_Ack",
        MessageType::OBJECT_REQUEST_NACK => "Object_Request_Nack",
        MessageType::DATA => "Data",
        MessageType::SERVER_STATUS => "Server_Status",
        MessageType::SERVER_STATUS_ACK => "Server_Status_Ack",
        MessageType::UNKNOWN => "Unknown",
    }
}

/**
 * This function convert a u8 to a MessageType
 *
 * @param value: u8, The source to convert
 *
 * @return MessageType
 */
impl From<u8> for MessageType {
    fn from(value: u8) -> Self {
        match value {
            0xF0 => MessageType::CONNECT,
            0xF1 => MessageType::CONNECT_ACK,
            0xF2 => MessageType::OPEN_STREAM,
            0xFF => MessageType::SHUTDOWN,
            0x01 => MessageType::HEARTBEAT,
            0x41 => MessageType::HEARTBEAT_REQUEST,
            0x02 => MessageType::PING,
            0x42 => MessageType::PONG,
            0x07 => MessageType::TOPIC_REQUEST,
            0x47 => MessageType::TOPIC_REQUEST_ACK,
            0x46 => MessageType::TOPIC_REQUEST_NACK,
            0x08 => MessageType::OBJECT_REQUEST,
            0x48 => MessageType::OBJECT_REQUEST_ACK,
            0x49 => MessageType::OBJECT_REQUEST_NACK,
            0x05 => MessageType::DATA,
            0x00 => MessageType::SERVER_STATUS,
            0x40 => MessageType::SERVER_STATUS_ACK,
            _ => MessageType::UNKNOWN
        }
    }
}

/**
 * This function convert a MessageType to an u8
 *
 * @param value: MessageType, The source to convert
 *
 * @return u8
 */
impl From<MessageType> for u8 {
    fn from(value: MessageType) -> Self {
        match value {
            MessageType::CONNECT => 0xF0,
            MessageType::CONNECT_ACK => 0xF1,
            MessageType::OPEN_STREAM => 0xF2,
            MessageType::SHUTDOWN => 0xFF,
            MessageType::HEARTBEAT => 0x01,
            MessageType::HEARTBEAT_REQUEST => 0x41,
            MessageType::PING => 0x02,
            MessageType::PONG => 0x42,
            MessageType::TOPIC_REQUEST => 0x07,
            MessageType::TOPIC_REQUEST_ACK => 0x47,
            MessageType::TOPIC_REQUEST_NACK => 0x46,
            MessageType::OBJECT_REQUEST => 0x08,
            MessageType::OBJECT_REQUEST_ACK => 0x48,
            MessageType::OBJECT_REQUEST_NACK => 0x49,
            MessageType::DATA => 0x05,
            MessageType::SERVER_STATUS => 0x00,
            MessageType::SERVER_STATUS_ACK => 0x40,
            MessageType::UNKNOWN => 0xAA,
        }
    }
}

/**
 * This function convert a LogLevel into a string
 *
 * @param loglevel: LogLevel, The source to convert
 *
 * @return String
 */
pub fn display_loglevel<'a>(loglevel: LogLevel) -> &'a str {
    match loglevel {
        LogLevel::All => "All",
        LogLevel::Info => "Info",
        LogLevel::Warning => "Warning",
        LogLevel::Error => "Error",
        LogLevel::Quiet => "Quiet",
    }
}

/**
 * ConnectStatus are all possible status
 * in a CONNECT_ACK request.
 */
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum ConnectStatus {
    SUCCESS,
    FAILURE,
    UNKNOWN,
}

/**
 * This function convert a ConnectStatus to an u8
 *
 * @param value: ConnectStatus, The source to convert
 *
 * @return u8
 */
impl From<ConnectStatus> for u8 {
    fn from(value: ConnectStatus) -> Self {
        match value {
            ConnectStatus::SUCCESS => 0x00,
            ConnectStatus::FAILURE => 0xFF,
            ConnectStatus::UNKNOWN => 0xAA,
        }
    }
}

/**
 * This function convert a u8 to a ConnectStatus
 *
 * @param value: u8, The source to convert
 *
 * @return ConnectStatus
 */
impl From<u8> for ConnectStatus {
    fn from(value: u8) -> Self {
        match value {
            0x00 => ConnectStatus::SUCCESS,
            0xFF => ConnectStatus::FAILURE,
            _ => ConnectStatus::UNKNOWN
        }
    }
}


/**
 * End connexion reasons are used to
 * detail the reason of the shutdown request.
 */
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum EndConnexionReason {
    SHUTDOWN,
    ERROR,
    UNKNOWN,
}

/**
 * This function convert a u8 to an EndConnexionReason
 *
 * @param value: u8, The source to convert
 *
 * @return EndConnexionReason
 */
impl From<u8> for EndConnexionReason {
    fn from(value: u8) -> Self {
        match value {
            0x00 => EndConnexionReason::SHUTDOWN,
            0x01 => EndConnexionReason::ERROR,
            _ => EndConnexionReason::UNKNOWN,
        }
    }
}

/**
 * This function convert an EndConnexionReason to an u8
 *
 * @param value: EndConnexionReason, The source to convert
 *
 * @return u8
 */
impl From<EndConnexionReason> for u8 {
    fn from(value: EndConnexionReason) -> Self {
        match value {
            EndConnexionReason::SHUTDOWN => 0x00,
            EndConnexionReason::ERROR => 0x01,
            EndConnexionReason::UNKNOWN => 0xAA
        }
    }
}

/**
 * Stream type are used to open stream
 * between the broker and a client.
 */
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum StreamType {
    MANAGEMENT,
    RELIABLE,
    UNRELIABLE,
    UNKNOWN,
}

/**
 * This function convert a StreamType to an u8
 *
 * @param value: StreamType, The source to convert
 *
 * @return u8
 */
impl From<u8> for StreamType {
    fn from(value: u8) -> Self {
        match value {
            0x00 => StreamType::MANAGEMENT,
            0x01 => StreamType::RELIABLE,
            0x02 => StreamType::UNRELIABLE,
            _ => StreamType::UNKNOWN
        }
    }
}

/**
 * This function convert an u8 to an StreamType
 *
 * @param value: u8, The source to convert
 *
 * @return StreamType
 */
impl From<StreamType> for u8 {
    fn from(value: StreamType) -> Self {
        match value {
            StreamType::MANAGEMENT => 0x00,
            StreamType::RELIABLE => 0x01,
            StreamType::UNRELIABLE => 0x02,
            StreamType::UNKNOWN => 0xAA,
        }
    }
}

/**
 * Topics action are all actions that
 * a peer can do in a TOPICS_REQUEST
 */
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum TopicsAction {
    SUBSCRIBE,
    UNSUBSCRIBE,
    UNKNOWN,
}

/**
 * This function convert a TopicsAction to an u8
 *
 * @param value: TopicsActions, The source to convert
 *
 * @return u8
 */
impl From<TopicsAction> for u8 {
    fn from(value: TopicsAction) -> Self {
        match value {
            TopicsAction::SUBSCRIBE => 0x00,
            TopicsAction::UNSUBSCRIBE => 0xFF,
            TopicsAction::UNKNOWN => 0xAA,
        }
    }
}

/**
 * This function convert an u8 to a TopicsActions
 *
 * @param value: u8, The source to convert
 *
 * @return TopicsActions
 */
impl From<u8> for TopicsAction {
    fn from(value: u8) -> Self {
        match value {
            0x00 => TopicsAction::SUBSCRIBE,
            0xFF => TopicsAction::UNSUBSCRIBE,
            _ => TopicsAction::UNKNOWN
        }
    }
}

/**
 * Topics response are all possible response
 * type to a TOPICS_REQUEST
 */
#[derive(Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum TopicsResponse {
    SUCCESS_SUB,
    FAILURE_SUB,
    SUCCESS_USUB,
    FAILURE_USUB,
    UNKNOWN,
}

/**
 * This function convert an TopicsResponse to an u8
 *
 * @param value: TopicsResponse, The source to convert
 *
 * @return u8
 */
impl From<TopicsResponse> for u8 {
    fn from(value: TopicsResponse) -> Self {
        match value {
            TopicsResponse::SUCCESS_SUB => 0x00,
            TopicsResponse::SUCCESS_USUB => 0x0F,
            TopicsResponse::FAILURE_SUB => 0xF0,
            TopicsResponse::FAILURE_USUB => 0xFF,
            TopicsResponse::UNKNOWN => 0xAA,
        }
    }
}

/**
 * This function convert an u8 to a TopicsResponse
 *
 * @param value: u8, The source to convert
 *
 * @return TopicsResponse
 */
impl From<u8> for TopicsResponse {
    fn from(value: u8) -> Self {
        match value {
            0x00 => TopicsResponse::SUCCESS_SUB,
            0x0F => TopicsResponse::SUCCESS_USUB,
            0xF0 => TopicsResponse::FAILURE_SUB,
            0xFF => TopicsResponse::FAILURE_USUB,
            _ => TopicsResponse::UNKNOWN
        }
    }
}

/**
 * Object identifier types are all possible source of identifier.
 * It Represent the twoMSB of the Object identifier : XX
 * 0--2------------64
 * |XX| IDENTIFIER |
 */
#[derive(Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum ObjectIdentifierType {
    USER_GENERATED,
    BROKER_GENERATED,
    TEMPORARY,
    UNKNOWN
}


/**
 * This function convert an ObjectIdentifierType to an u8
 *
 * @param value: ObjectIdentifierType, The source to convert
 *
 * @return u8
 */
impl From<ObjectIdentifierType> for u8 {
    fn from(value: ObjectIdentifierType) -> Self {
        match value {
            ObjectIdentifierType::USER_GENERATED => 0x00,
            ObjectIdentifierType::BROKER_GENERATED => 0x01,
            ObjectIdentifierType::TEMPORARY => 0x10,
            ObjectIdentifierType::UNKNOWN => 0xAA,
        }
    }
}

/**
 * This function convert an u8 to a ObjectIdentifierType
 *
 * @param value: u8, The source to convert
 *
 * @return ObjectIdentifierType
 */
impl From<u8> for ObjectIdentifierType {
    fn from(value: u8) -> Self {
        match value {
             0x00 => ObjectIdentifierType::USER_GENERATED,
             0x01 => ObjectIdentifierType::BROKER_GENERATED,
             0x10 => ObjectIdentifierType::TEMPORARY,
             _ => ObjectIdentifierType::UNKNOWN,
        }
    }
}


/**
 * Object Flags are all possible action in a
 * OBJECT_REQUEST.
 */
#[derive(Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum ObjectFlags {
    CREATE,
    UPDATE,
    DELETE,
    SUBSCRIBE,
    UNSUBSCRIBE,
    UNKNOWN,
}


/**
 * This function convert an ObjectFlags to an u8
 *
 * @param value: ObjectFlags, The source to convert
 *
 * @return u8
 */
impl From<ObjectFlags> for u8 {
    fn from(value: ObjectFlags) -> Self {
        match value {
            ObjectFlags::CREATE => 0x01,
            ObjectFlags::UPDATE => 0x02,
            ObjectFlags::DELETE => 0x04,
            ObjectFlags::SUBSCRIBE => 0x08,
            ObjectFlags::UNSUBSCRIBE => 0x10,
            ObjectFlags::UNKNOWN => 0xAA,
        }
    }
}

/**
 * This function convert an u8 to a ObjectFlags
 *
 * @param value: u8, The source to convert
 *
 * @return ObjectFlags
 */
impl From<u8> for ObjectFlags {
    fn from(value: u8) -> Self {
        match value {
            0x01 => ObjectFlags::CREATE,
            0x02 => ObjectFlags::UPDATE,
            0x04 => ObjectFlags::DELETE,
            0x08 => ObjectFlags::SUBSCRIBE,
            0x10 => ObjectFlags::UNSUBSCRIBE,
            _ => ObjectFlags::UNKNOWN,
        }
    }
}

/**
 * This functions return a bytes slice according to
 * the given bounds. FROM and TO are include in the returned slice.
 *
 * @param buffer: &[u8], the original array,
 * @param from: usize, first bound,
 * @param to: usize, last bound,
 *
 * @return Vec<u8>, the slice requested
 */
pub fn get_bytes_from_slice(
    buffer: &[u8],
    from: usize,
    to: usize
) -> Vec<u8> {
    // 1 - check bound validity
    match () {
        _ if to < from => panic!("from is greater than to"),
        _ if to >= buffer.len() => panic!("to is greater than the last index"),
        _ => (),
    }

    // 2 - return the correct slice
    buffer[from..to+1].into()
}

/** ==================================
*
*              Datagrams
*
** ================================*/

//===== Sent to know the server status
pub struct RQ_ServerStatus {
    pub message_type: MessageType,
}
impl RQ_ServerStatus {
    pub const fn new() -> RQ_ServerStatus {
        RQ_ServerStatus { message_type: MessageType::SERVER_STATUS }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        return [u8::from(self.message_type)].into();
    }
}

impl From<&[u8]> for RQ_ServerStatus {
    fn from(buffer: &[u8]) -> Self {
        RQ_ServerStatus {
            message_type: MessageType::from(*buffer.first().unwrap()),
        }
    }
}
//===== Sent to answer a ServerStatus request
pub struct RQ_ServerStatus_ACK {
    pub message_type: MessageType,
    pub status: bool,
    pub connected_client: ClientId, // Amount of connected client. It use the same type as client_id to ensure sufficient capacity
}
impl RQ_ServerStatus_ACK {
    pub const fn new(status: bool, nb_client: ClientId) -> RQ_ServerStatus_ACK {
        RQ_ServerStatus_ACK {
            message_type: MessageType::SERVER_STATUS_ACK,
            status,
            connected_client: nb_client
        }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(10);
        bytes.push(u8::from(self.message_type));
        bytes.push(u8::from(self.status));
        bytes.extend(self.connected_client.to_le_bytes());

        return bytes;
    }
}

impl From<&[u8]> for RQ_ServerStatus_ACK {
    fn from(buffer: &[u8]) -> Self {
        RQ_ServerStatus_ACK {
            message_type: MessageType::from(*buffer.first().unwrap()),
            status: buffer[1] != 0,
            connected_client: u64::from_le_bytes(buffer[2..].split_at(size_of::<u64>()).0.try_into().unwrap()),
        }
    }
}

//===== Sent to connect to a peer to the server
pub struct RQ_Connect {
    pub message_type: MessageType,
}
impl RQ_Connect {
    pub const fn new() -> RQ_Connect {
        RQ_Connect { message_type: MessageType::CONNECT }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        return [u8::from(self.message_type)].into();
    }
}

impl From<&[u8]> for RQ_Connect {
    fn from(buffer: &[u8]) -> Self {
        RQ_Connect {
            message_type: MessageType::from(*buffer.first().unwrap()),
        }
    }
}


//===== Sent to acknowledge the connexion
pub struct RQ_Connect_ACK_OK {
    pub message_type: MessageType,
    pub status: ConnectStatus,
    pub peer_id: u64,
    pub heartbeat_period: u16,
}

impl RQ_Connect_ACK_OK {
    pub const fn new(peer_id: u64, heartbeat_period: u16) -> RQ_Connect_ACK_OK {
        RQ_Connect_ACK_OK { message_type: MessageType::CONNECT_ACK, status: ConnectStatus::SUCCESS, peer_id, heartbeat_period }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(12);
        bytes.push(u8::from(self.message_type));
        bytes.push(u8::from(self.status));
        bytes.extend(self.peer_id.to_le_bytes().into_iter());
        bytes.extend(self.heartbeat_period.to_le_bytes().into_iter());

        return bytes;
    }
}

impl From<&[u8]> for RQ_Connect_ACK_OK {
    fn from(buffer: &[u8]) -> Self {
        RQ_Connect_ACK_OK {
            message_type: MessageType::CONNECT_ACK,
            status: ConnectStatus::SUCCESS,
            peer_id: u64::from_le_bytes(get_bytes_from_slice(buffer, 2, 9).try_into().expect("Cannot get the peer_id slice from the buffer")),
            heartbeat_period: u16::from_le_bytes(get_bytes_from_slice(buffer, 10, 11).try_into().expect("Cannot get the heartbeat_period slice from the buffer")),
        }
    }
}

pub struct RQ_Connect_ACK_ERROR {
    pub message_type: MessageType,
    pub status: ConnectStatus,
    pub message_size: Size,
    pub reason: Vec<u8>,
}

impl RQ_Connect_ACK_ERROR {
    pub fn new(message: &str) -> RQ_Connect_ACK_ERROR {
        let reason: Vec<u8> = message.as_bytes().into();
        let message_size = (reason.len() + 1) as u16;
        RQ_Connect_ACK_ERROR { message_type: MessageType::CONNECT_ACK, status: ConnectStatus::FAILURE, message_size, reason }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(4 + self.reason.len());
        bytes.push(u8::from(self.message_type));
        bytes.extend(self.message_size.to_le_bytes().into_iter());
        bytes.push(u8::from(self.status));
        bytes.extend(&mut self.reason.iter());

        return bytes;
    }
}

impl From<&[u8]> for RQ_Connect_ACK_ERROR {
    fn from(buffer: &[u8]) -> Self {
        let size = u16::from_le_bytes(get_bytes_from_slice(buffer, 1, 2).try_into().expect("Cannot get size from buffer."));
        RQ_Connect_ACK_ERROR {
            message_type: MessageType::CONNECT_ACK,
            status: ConnectStatus::FAILURE,
            message_size: size,
            reason: get_bytes_from_slice(buffer, 4, (4 + size - 1) as usize),
        }
    }
}

//===== Sent to maintain the connexion
pub struct RQ_Heartbeat {
    pub message_type: MessageType,
}
impl RQ_Heartbeat {
    pub const fn new() -> RQ_Heartbeat {
        RQ_Heartbeat { message_type: MessageType::HEARTBEAT }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        return [u8::from(self.message_type)].into();
    }
}

impl From<&[u8]> for RQ_Heartbeat {
    fn from(_buffer: &[u8]) -> Self {
        RQ_Heartbeat {
            message_type: MessageType::HEARTBEAT
        }
    }
}

//===== Sent to request a Heartbeat if a pear do not receive his
// normal heartbeat.
pub struct RQ_Heartbeat_Request {
    pub message_type: MessageType,
}
impl RQ_Heartbeat_Request {
    pub const fn new() -> RQ_Heartbeat_Request {
        RQ_Heartbeat_Request { message_type: MessageType::HEARTBEAT_REQUEST }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        return [u8::from(self.message_type)].into();
    }
}

impl From<&[u8]> for RQ_Heartbeat_Request {
    fn from(_buffer: &[u8]) -> Self {
        RQ_Heartbeat_Request {
            message_type: MessageType::HEARTBEAT_REQUEST
        }
    }
}

//===== Sent to measure the latency between peer and broker
pub struct RQ_Ping {
    pub message_type: MessageType,
    pub ping_id: PingId,
}

impl RQ_Ping {
    pub const fn new(ping_id: PingId) -> RQ_Ping {
        RQ_Ping { message_type: MessageType::PING, ping_id }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(2);
        bytes.push(u8::from(self.message_type));
        bytes.push(self.ping_id);
        return bytes;
    }
}

impl From<&[u8]> for RQ_Ping {
    fn from(buffer: &[u8]) -> Self {
        RQ_Ping {
            message_type: MessageType::PING,
            ping_id: *buffer.get(1).unwrap(),
        }
    }
}

//===== Sent to answer a ping request.
pub struct RQ_Pong {
    pub message_type: MessageType,
    pub ping_id: PingId,
}

impl RQ_Pong {
    pub const fn new(ping_id: PingId) -> RQ_Ping {
        RQ_Ping { message_type: MessageType::PONG, ping_id }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(2);
        bytes.push(u8::from(self.message_type));
        bytes.push(self.ping_id);
        return bytes;
    }
}

impl From<&[u8]> for RQ_Pong {
    fn from(buffer: &[u8]) -> Self {
        RQ_Pong {
            message_type: MessageType::PONG,
            ping_id: *buffer.get(1).unwrap(),
        }
    }
}

//===== Sent to close the connexion between peer and broker
pub struct RQ_Shutdown {
    pub message_type: MessageType,
    pub reason: EndConnexionReason,
}

impl RQ_Shutdown {
    pub const fn new(reason: EndConnexionReason) -> RQ_Shutdown {
        RQ_Shutdown { message_type: MessageType::SHUTDOWN, reason }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(2);
        bytes.push(u8::from(self.message_type));
        bytes.push(u8::from(self.reason));
        return bytes;
    }
}

impl From<&[u8]> for RQ_Shutdown {
    fn from(buffer: &[u8]) -> Self {
        RQ_Shutdown {
            message_type: MessageType::SHUTDOWN,
            reason: EndConnexionReason::from(*buffer.get(1).unwrap()),
        }
    }
}

//===== Sent to open a new stream
pub struct RQ_OpenStream {
    pub message_type: MessageType,
    pub stream_type: StreamType,
}

impl RQ_OpenStream {
    pub const fn new(stream_type: StreamType) -> RQ_OpenStream {
        RQ_OpenStream { message_type: MessageType::OPEN_STREAM, stream_type }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(2);
        bytes.push(u8::from(self.message_type));
        bytes.push(u8::from(self.stream_type));
        return bytes;
    }
}

impl From<&[u8]> for RQ_OpenStream {
    fn from(buffer: &[u8]) -> Self {
        RQ_OpenStream {
            message_type: MessageType::OPEN_STREAM,
            stream_type: StreamType::from(buffer.get(1).unwrap().clone()),
        }
    }
}

//===== Sent to subscribe/unsubscribe to a topic
pub struct RQ_TopicRequest {
    pub message_type: MessageType, // 1 byte
    pub action: TopicsAction, // 1 byte
    pub size: Size, // 2 bytes (u16)
    pub topic_id: u64, // size bytes
}

//===== Sent to subscribe a topic
impl RQ_TopicRequest {
    pub fn new(action: TopicsAction, topic_id: u64) -> RQ_TopicRequest {
        let size = (8 + 1) as u16; // topic_id + 1 for the action
        RQ_TopicRequest { message_type: MessageType::TOPIC_REQUEST, action, size, topic_id }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(12);
        bytes.push(u8::from(self.message_type));
        bytes.extend(self.size.to_le_bytes());
        bytes.push(u8::from(self.action));
        bytes.extend(self.topic_id.to_le_bytes());
        return bytes;
    }
}

impl From<&[u8]> for RQ_TopicRequest {
    fn from(buffer: &[u8]) -> Self {
        let size = u16::from_le_bytes(buffer[1..].split_at(size_of::<u16>()).0.try_into().unwrap());
        let payload_end = 4 + (size - 1) as usize;

        RQ_TopicRequest {
            message_type: MessageType::TOPIC_REQUEST,
            action: TopicsAction::from(*buffer.get(3).unwrap()),
            size,
            topic_id: u64::from_le_bytes(get_bytes_from_slice(buffer,4,11).try_into().expect("Failed to get the topic id slice from the buffer in a RQ_TopicRequest from u8")),
        }
    }
}

//===== Sent to acknowledge a TOPIC_REQUEST
pub struct RQ_TopicRequest_ACK {
    pub message_type: MessageType,
    pub status: TopicsResponse,
    pub topic_id: u64,
}

impl RQ_TopicRequest_ACK {
    pub const fn new(topic_id: u64, status: TopicsResponse) -> RQ_TopicRequest_ACK {
        RQ_TopicRequest_ACK { message_type: MessageType::TOPIC_REQUEST_ACK, status, topic_id }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(10);
        bytes.push(u8::from(self.message_type));
        bytes.push(u8::from(self.status));
        bytes.extend(self.topic_id.to_le_bytes());
        return bytes;
    }
}

impl From<&[u8]> for RQ_TopicRequest_ACK {
    fn from(buffer: &[u8]) -> Self {
        RQ_TopicRequest_ACK {
            message_type: MessageType::TOPIC_REQUEST_ACK,
            status: TopicsResponse::from(*buffer.get(1).unwrap()),
            topic_id: u64::from_le_bytes(get_bytes_from_slice(buffer, 2, 9).try_into().expect("Failed to get the topic id slice from the buffer a RQ_TopicRequest_ACK from u8")),
        }
    }
}

pub struct RQ_TopicRequest_NACK {
    pub message_type: MessageType,
    pub status: TopicsResponse,
    pub size: Size,
    pub error_message: Vec<u8>
}

impl RQ_TopicRequest_NACK{

    pub fn new(status: TopicsResponse, error_message: &str) -> RQ_TopicRequest_NACK {
        let size = (error_message.len() + 1) as u16; // string length + 1 for the action
        RQ_TopicRequest_NACK { message_type: MessageType::TOPIC_REQUEST_ACK, status, size, error_message: error_message.as_bytes().into()}
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(2 + self.error_message.len());
        bytes.push(u8::from(self.message_type));
        bytes.push(u8::from(self.status));
        bytes.extend(self.error_message.iter());

        return bytes;
    }
}

impl From<&[u8]> for RQ_TopicRequest_NACK {
    fn from(buffer: &[u8]) -> Self {
        let size = u16::from_le_bytes(get_bytes_from_slice(buffer, 2, 3).try_into().expect("Bad Size received a RQ_TopicRequest_NACK from u8"));
        RQ_TopicRequest_NACK {
            message_type: MessageType::TOPIC_REQUEST_ACK,
            status: TopicsResponse::from(buffer.get(1).unwrap().clone()),
            size,
            error_message: get_bytes_from_slice(buffer, 4, (4 + size - 1) as usize)
        }
    }
}


pub struct RQ_Data{
    pub message_type: MessageType, // 1 byte
    pub size: Size, // 2 bytes (u16)
    pub sequence_number: u32, // 4 bytes (u32)
    pub topic_id: u64, // 8 bytes (u64)
    pub data: Vec<u8> // size bytes
}
impl RQ_Data {

    pub fn new(sequence_number: u32, topic_id: u64, payload: Vec<u8>)-> RQ_Data {
        let size = (payload.len() + 8) as u16; // payload length + 8 for topic id
        RQ_Data { message_type: MessageType::DATA, size, sequence_number, topic_id, data: payload }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(15 + self.data.len());
        bytes.push(u8::from(self.message_type));
        bytes.extend(self.size.to_le_bytes());
        bytes.extend(self.sequence_number.to_le_bytes());
        bytes.extend(self.topic_id.to_le_bytes());
        bytes.extend(self.data.iter());
        return bytes;
    }
}

impl From<&[u8]> for RQ_Data {
    fn from(buffer: &[u8]) -> Self {
        let size = u16::from_le_bytes(buffer[1..].split_at(size_of::<u16>()).0.try_into().unwrap());
        let data_end = 15 + (size - 8) as usize;

        RQ_Data {
            message_type: MessageType::DATA,
            size,
            sequence_number: u32::from_le_bytes(buffer[3..].split_at(size_of::<u32>()).0.try_into().unwrap()),
            topic_id: u64::from_le_bytes(buffer[7..].split_at(size_of::<u64>()).0.try_into().unwrap()),
            data: buffer[15..data_end].into(),
        }
    }
}


//===== Sent to acknowledge a TOPIC_REQUEST
pub struct RQ_ObjectRequest{
    pub message_type: MessageType,
    pub size: Size,
    pub flags: ObjectFlags,
    pub object_id: ObjectId,
    pub topics: HashSet<u64>
}
impl RQ_ObjectRequest {
    pub fn new(flags: ObjectFlags, object_id: u64, topics: HashSet<u64>)-> RQ_ObjectRequest {
        let size: u16 = (1 + 8 + topics.len() * size_of::<u64>()) as u16; // 1 = flags, 8 = object_id, x = size_of(topics)
        RQ_ObjectRequest{
            message_type:MessageType::OBJECT_REQUEST,
            size,
            flags,
            object_id,
            topics,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(3 + self.size as usize); // 3 = messageType + size
        bytes.push(u8::from(self.message_type));
        bytes.extend(self.size.to_le_bytes());
        bytes.push(u8::from(self.flags));
        bytes.extend(self.object_id.to_le_bytes());
        // The following line convert a Vec<u64> to his representation as bytes (Vec<u8>)
        bytes.extend(self.topics.iter()
                         .flat_map(|&x| {
                             let bytes: [u8; 8] = x.to_le_bytes();
                             bytes.into_iter()
                         })
                         .collect::<Vec<u8>>());
        return bytes;
    }
}

impl From<&[u8]> for RQ_ObjectRequest {
    fn from(buffer: &[u8]) -> Self {
        let size = u16::from_le_bytes(buffer[1..].split_at(size_of::<u16>()).0.try_into().unwrap());
        let topics: HashSet<u64> = get_bytes_from_slice(buffer, 12, (size as usize - 9))
            // Convert the bytes vector to a vector of topics id by grouping u8 into u64
            .chunks_exact(8)
            .map(|chunk| {
                u64::from_le_bytes(chunk.try_into().unwrap())
            })
            .collect();

        RQ_ObjectRequest {
            message_type: MessageType::OBJECT_REQUEST,
            size,
            flags: ObjectFlags::from(buffer[3]),
            object_id: u64::from_le_bytes(buffer[4..].split_at(size_of::<u64>()).0.try_into().unwrap()),
            topics,
        }
    }
}

//===== Sent to acknowledge a OBJECT_REQUEST create
pub struct RQ_ObjectRequestCreate_ACK{
    pub message_type: MessageType,
    pub flags: u8, // Bit field SXXX XDMC (S: Success/fail, X: Unused, D: delete, M : modify, C: Create)
    pub old_object_id: u64,
    pub final_object_id: u64
}
impl RQ_ObjectRequestCreate_ACK {
    pub fn new(flags: u8, old_id: u64, new_id: u64)-> RQ_ObjectRequestCreate_ACK {
        RQ_ObjectRequestCreate_ACK{
            message_type:MessageType::OBJECT_REQUEST_ACK,
            flags,
            old_object_id: old_id,
            final_object_id: new_id
        }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(18);
        bytes.push(u8::from(self.message_type));
        bytes.push(u8::from(self.flags));
        bytes.extend(self.old_object_id.to_le_bytes());
        bytes.extend(self.final_object_id.to_le_bytes());

        return bytes;
    }
}

impl From<&[u8]> for RQ_ObjectRequestCreate_ACK {
    fn from(buffer: &[u8]) -> Self {
        RQ_ObjectRequestCreate_ACK {
            message_type: MessageType::OBJECT_REQUEST_ACK,
            flags: buffer[1],
            old_object_id: u64::from_le_bytes(buffer[2..].split_at(size_of::<u64>()).0.try_into().unwrap()),
            final_object_id: u64::from_le_bytes(buffer[10..].split_at(size_of::<u64>()).0.try_into().unwrap()),
        }
    }
}

//===== Sent to acknowledge a OBJECT_REQUEST delete or update
pub struct RQ_ObjectRequestDefault_ACK{
    pub message_type: MessageType,
    pub flags: u8, // Bit field SXXA UDMC (S: Success/fail, X: Unused, D: delete, M : modify, C: Create, A : subscribe, U, Unsubscribe)
    pub object_id: u64,
}
impl RQ_ObjectRequestDefault_ACK {
    pub fn new(flags: u8, old_id: u64)-> RQ_ObjectRequestDefault_ACK {
        RQ_ObjectRequestDefault_ACK{
            message_type:MessageType::OBJECT_REQUEST_ACK,
            flags,
            object_id: old_id,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(18);
        bytes.push(u8::from(self.message_type));
        bytes.push(u8::from(self.flags));
        bytes.extend(self.object_id.to_le_bytes());

        return bytes;
    }
}

impl From<&[u8]> for RQ_ObjectRequestDefault_ACK {
    fn from(buffer: &[u8]) -> Self {
        RQ_ObjectRequestDefault_ACK {
            message_type: MessageType::OBJECT_REQUEST_ACK,
            flags: buffer[1],
            object_id: u64::from_le_bytes(buffer[2..].split_at(size_of::<u64>()).0.try_into().unwrap()),
        }
    }
}




// ===== Sent in case of error for all action (Create update delete)
pub struct RQ_ObjectRequest_NACK{
    pub message_type: MessageType,
    pub flags: u8, // Bit field SXXA UDMC (S: Success/fail, X: Unused, D: delete, M : modify, C: Create, A : subscribe, U, Unsubscribe)
    pub object_id: u64,
    pub reason_size: Size,
    pub reason: Vec<u8>
}
impl RQ_ObjectRequest_NACK {
    pub fn new(flags: u8, object_id: u64, reason: &str)-> RQ_ObjectRequest_NACK {
        let reason_vec: Vec<u8> = reason.as_bytes().into();
        RQ_ObjectRequest_NACK{
            message_type:MessageType::OBJECT_REQUEST_NACK,
            flags,
            object_id,
            reason_size: reason_vec.len() as Size,
            reason: reason_vec
        }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(12 + self.reason_size as usize);
        bytes.push(u8::from(self.message_type));
        bytes.push(u8::from(self.flags));
        bytes.extend(self.object_id.to_le_bytes());
        bytes.extend(self.reason_size.to_le_bytes());
        bytes.extend(self.reason.iter());

        return bytes;
    }
}

impl From<&[u8]> for RQ_ObjectRequest_NACK {
    fn from(buffer: &[u8]) -> Self {
        let size: Size = u16::from_le_bytes(buffer[10..].split_at(size_of::<u16>()).0.try_into().unwrap());
        RQ_ObjectRequest_NACK {
            message_type: MessageType::OBJECT_REQUEST_NACK,
            flags: buffer[1],
            object_id: u64::from_le_bytes(buffer[2..].split_at(size_of::<u64>()).0.try_into().unwrap()),
            reason_size: size,
            reason: get_bytes_from_slice(buffer, 12, (size - 1) as usize)
        }
    }
}