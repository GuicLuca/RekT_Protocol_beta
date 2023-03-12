#![allow(non_camel_case_types)]

use std::fmt;
use std::fmt::Display;

use crate::ps_common::get_bytes_from_slice;

/** ==================================
*
*      ENUMERATIONS STRUCT &
*         GLOBAL CLASSES
*
** ================================*/

// Message type are used to translate
// request type to the corresponding code
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
    OBJECT_REQUEST,
    OBJECT_REQUEST_ACK,
    DATA,
    UNKNOWN,
}

pub fn display(message: MessageType) -> String {
    match message {
        MessageType::CONNECT => "Connect".to_string(),
        MessageType::CONNECT_ACK => "Connect_ACK".to_string(),
        MessageType::OPEN_STREAM => "Open_Stream".to_string(),
        MessageType::SHUTDOWN => "Shutdown".to_string(),
        MessageType::HEARTBEAT => "HeartBeat".to_string(),
        MessageType::HEARTBEAT_REQUEST => "HeartBeat_Request".to_string(),
        MessageType::PING => "Ping".to_string(),
        MessageType::PONG => "Pong".to_string(),
        MessageType::TOPIC_REQUEST => "Topic_Request".to_string(),
        MessageType::TOPIC_REQUEST_ACK => "Topic_Request_Ack".to_string(),
        MessageType::OBJECT_REQUEST => "Object_Request".to_string(),
        MessageType::OBJECT_REQUEST_ACK => "Object_Request_Ack".to_string(),
        MessageType::DATA => "Data".to_string(),
        MessageType::UNKNOWN => "Unknown".to_string()
    }
}

impl Eq for MessageType {}

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
            0x08 => MessageType::OBJECT_REQUEST,
            0x48 => MessageType::OBJECT_REQUEST_ACK,
            0x05 => MessageType::DATA,
            _ => MessageType::UNKNOWN
        }
    }
}

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
            MessageType::OBJECT_REQUEST => 0x08,
            MessageType::OBJECT_REQUEST_ACK => 0x48,
            MessageType::DATA => 0x05,
            MessageType::UNKNOWN => 0xAA
        }
    }
}


// Connect status are all possible status
// in a CONNECT_ACK request
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum ConnectStatus {
    SUCCESS,
    FAILURE,
    UNKNOWN,
}

impl From<ConnectStatus> for u8 {
    fn from(value: ConnectStatus) -> Self {
        match value {
            ConnectStatus::SUCCESS => 0x00,
            ConnectStatus::FAILURE => 0xFF,
            ConnectStatus::UNKNOWN => 0xAA,
        }
    }
}

impl From<u8> for ConnectStatus {
    fn from(value: u8) -> Self {
        match value {
            0x00 => ConnectStatus::SUCCESS,
            0xFF => ConnectStatus::FAILURE,
            _ => ConnectStatus::UNKNOWN
        }
    }
}


// End connexion reasons are used to
// detail the reason of the shutdown request.
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum EndConnexionReason {
    SHUTDOWN,
    ERROR,
    UNKNOWN,
}

impl From<u8> for EndConnexionReason {
    fn from(value: u8) -> Self {
        match value {
            0x00 => EndConnexionReason::SHUTDOWN,
            0x01 => EndConnexionReason::ERROR,
            _ => EndConnexionReason::UNKNOWN,
        }
    }
}

impl From<EndConnexionReason> for u8 {
    fn from(value: EndConnexionReason) -> Self {
        match value {
            EndConnexionReason::SHUTDOWN => 0x00,
            EndConnexionReason::ERROR => 0x01,
            EndConnexionReason::UNKNOWN => 0xAA
        }
    }
}

// End connexion reasons are used to
// detail the reason of the shutdown request.
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum StreamType {
    MANAGEMENT,
    RELIABLE,
    UNRELIABLE,
    UNKNOWN,
}

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

// Topics action are all actions that
// a peer can do in a TOPICS_REQUEST
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum TopicsAction {
    SUBSCRIBE,
    UNSUBSCRIBE,
    UNKNOWN,
}

impl From<TopicsAction> for u8 {
    fn from(value: TopicsAction) -> Self {
        match value {
            TopicsAction::SUBSCRIBE => 0x00,
            TopicsAction::UNSUBSCRIBE => 0xFF,
            TopicsAction::UNKNOWN => 0xAA,
        }
    }
}

impl From<u8> for TopicsAction {
    fn from(value: u8) -> Self {
        match value {
            0x00 => TopicsAction::SUBSCRIBE,
            0xFF => TopicsAction::UNSUBSCRIBE,
            _ => TopicsAction::UNKNOWN
        }
    }
}

// Topics response are all possible response
// type to a TOPICS_REQUEST
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum TopicsResponse {
    SUCCESS_SUB,
    FAILURE_SUB,
    SUCCESS_USUB,
    FAILURE_USUB,
    UNKNOWN,
}

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

#[derive(Clone, Copy)]
pub struct Size {
    pub size: u16,
}
impl Size {
    pub const fn new(size: u16) -> Size {
        Size {
            size,
        }
    }
}

/** ==================================
*
*              Datagrams
*
** ================================*/

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
        let bytes = [u8::from(self.message_type)].to_vec();

        return bytes;
    }
}

impl From<&[u8]> for RQ_Connect {
    fn from(value: &[u8]) -> Self {
        RQ_Connect {
            message_type: MessageType::from(value.first().unwrap().clone()),
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
        let mut bytes = [u8::from(self.message_type)].to_vec();
        bytes.append(&mut [u8::from(self.status)].to_vec());
        bytes.append(&mut self.peer_id.to_le_bytes().to_vec());
        bytes.append(&mut self.heartbeat_period.to_le_bytes().to_vec());

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
        let reason = message.as_bytes().to_vec();
        let message_size = Size::new((reason.len() + 1) as u16);
        RQ_Connect_ACK_ERROR { message_type: MessageType::CONNECT_ACK, status: ConnectStatus::FAILURE, message_size, reason }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes = [u8::from(self.message_type)].to_vec();
        bytes.append(&mut self.message_size.size.to_le_bytes().to_vec());
        bytes.append(&mut [u8::from(self.status)].to_vec());
        bytes.append(&mut self.reason.clone());

        return bytes;
    }
}

impl From<&[u8]> for RQ_Connect_ACK_ERROR {
    fn from(buffer: &[u8]) -> Self {
        let size = u16::from_le_bytes(get_bytes_from_slice(buffer, 1, 2).try_into().expect("Cannot get size from buffer."));
        RQ_Connect_ACK_ERROR {
            message_type: MessageType::CONNECT_ACK,
            status: ConnectStatus::FAILURE,
            message_size: Size::new(size),
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
        let mut bytes = [u8::from(self.message_type)].to_vec();

        return bytes;
    }
}

impl From<&[u8]> for RQ_Heartbeat {
    fn from(_buffer: &[u8]) -> Self {
        RQ_Heartbeat {
            message_type: MessageType::HEARTBEAT
        }
    }
}

//===== Sent to request a Heartbeat if a pear do not recive his
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
        let mut bytes = [u8::from(self.message_type)].to_vec();

        return bytes;
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
    pub ping_id: u8,
}

impl RQ_Ping {
    pub const fn new(ping_id: u8) -> RQ_Ping {
        RQ_Ping { message_type: MessageType::PING, ping_id }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes = [u8::from(self.message_type)].to_vec();
        bytes.push(self.ping_id);

        return bytes;
    }
}

impl From<&[u8]> for RQ_Ping {
    fn from(buffer: &[u8]) -> Self {
        RQ_Ping {
            message_type: MessageType::PING,
            ping_id: buffer.get(1).unwrap().clone(),
        }
    }
}

//===== Sent to answer a ping request.
pub struct RQ_Pong {
    pub message_type: MessageType,
    pub ping_id: u8,
}

impl RQ_Pong {
    pub const fn new(ping_id: u8) -> RQ_Ping {
        RQ_Ping { message_type: MessageType::PONG, ping_id }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes = [u8::from(self.message_type)].to_vec();
        bytes.push(self.ping_id);

        return bytes;
    }
}

impl From<&[u8]> for RQ_Pong {
    fn from(buffer: &[u8]) -> Self {
        RQ_Pong {
            message_type: MessageType::PONG,
            ping_id: buffer.get(1).unwrap().clone(),
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
        let mut bytes = [u8::from(self.message_type)].to_vec();
        bytes.append(&mut [u8::from(self.reason)].to_vec());

        return bytes;
    }
}

impl From<&[u8]> for RQ_Shutdown {
    fn from(buffer: &[u8]) -> Self {
        RQ_Shutdown {
            message_type: MessageType::SHUTDOWN,
            reason: EndConnexionReason::from(buffer.get(1).unwrap().clone()),
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
        let mut bytes = [u8::from(self.message_type)].to_vec();
        bytes.append(&mut [u8::from(self.stream_type)].to_vec());

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
    pub message_type: MessageType,
    pub action: TopicsAction,
    pub size: Size,
    pub payload: Vec<u8>,
}

impl RQ_TopicRequest {
    pub fn new(action: TopicsAction, payload: &str) -> RQ_TopicRequest {
        let payload = payload.as_bytes().to_vec();
        let size = Size::new((payload.len() + 1) as u16); // string length + 1 for the action
        RQ_TopicRequest { message_type: MessageType::TOPIC_REQUEST, action, size, payload }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes = [u8::from(self.message_type)].to_vec();
        bytes.append(&mut self.size.size.to_le_bytes().to_vec());
        bytes.append(&mut [u8::from(self.action)].to_vec());
        bytes.append(&mut self.payload.clone());

        return bytes;
    }
}

impl From<&[u8]> for RQ_TopicRequest {
    fn from(buffer: &[u8]) -> Self {
        let size = Size::new(u16::from_le_bytes(get_bytes_from_slice(buffer, 1, 2).try_into().expect("Bad Size recieved")));
        let payload_end = 4 + (size.size - 1) as usize;

        RQ_TopicRequest {
            message_type: MessageType::TOPIC_REQUEST,
            action: TopicsAction::from(buffer.get(3).unwrap().clone()),
            size,
            payload: buffer[4..payload_end].to_vec(),
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
        let mut bytes = [u8::from(self.message_type)].to_vec();
        bytes.append(&mut [u8::from(self.status)].to_vec());
        bytes.append(&mut self.topic_id.to_le_bytes().to_vec());

        return bytes;
    }
}

impl From<&[u8]> for RQ_TopicRequest_ACK {
    fn from(buffer: &[u8]) -> Self {
        RQ_TopicRequest_ACK {
            message_type: MessageType::TOPIC_REQUEST_ACK,
            status: TopicsResponse::from(buffer.get(1).unwrap().clone()),
            topic_id: u64::from_le_bytes(get_bytes_from_slice(buffer, 2, 9).to_vec().try_into().expect("Failed to get the topic id slice from the buffer")),
        }
    }
}

pub struct RQ_TopicRequest_NACK {
    pub message_type: MessageType,
    pub status: TopicsResponse,
    pub size: Size,
    pub error_message: String
    // black magic
}

impl RQ_TopicRequest_NACK {

    pub fn new(status: TopicsResponse, error_message: String) -> RQ_TopicRequest_NACK {
        let size = Size::new((error_message.len() + 1) as u16); // string length + 1 for the action
        RQ_TopicRequest_NACK { message_type: MessageType::TOPIC_REQUEST_ACK, status, size, error_message}
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes = [u8::from(self.message_type)].to_vec();
        bytes.append(&mut [u8::from(self.status)].to_vec());
        bytes.append(&mut self.error_message.to_string().as_bytes().to_vec());

        return bytes;
    }
}

impl From<&[u8]> for RQ_TopicRequest_NACK {
    fn from(buffer: &[u8]) -> Self {
        let size = Size::new(u16::from_le_bytes(get_bytes_from_slice(buffer, 2, 3).try_into().expect("Bad Size recieved")));
        RQ_TopicRequest_NACK {
            message_type: MessageType::TOPIC_REQUEST_ACK,
            status: TopicsResponse::from(buffer.get(1).unwrap().clone()),
            size,
            error_message: String::from_utf8(get_bytes_from_slice(buffer, 4, (4 + size.size - 1) as usize)).unwrap()
        }
    }
}


/*
//===== Sent to acknowledge a TOPIC_REQUEST
pub struct RQ_ObjectRequest{
    pub message_type: MessageType,
    pub status: TopicsResponse,
    pub topic_id: u64
}
impl RQ_ObjectRequest {
    pub const fn new(status: TopicsResponse, topic_id: u64)-> RQ_ObjectRequest {
        RQ_ObjectRequest{message_type:MessageType::OBJECT_REQUEST, status, topic_id}
    }
}

//===== Sent to acknowledge a TOPIC_REQUEST
pub struct RQ_ObjectRequest_ACK{
    pub message_type: MessageType,
    pub status: TopicsResponse,
    pub topic_id: u64
}
impl RQ_ObjectRequest_ACK {
    pub const fn new(status: TopicsResponse, topic_id: u64)-> RQ_ObjectRequest_ACK {
        RQ_ObjectRequest_ACK{message_type:MessageType::OBJECT_REQUEST_ACK, status, topic_id}
    }
}

//===== Sent to acknowledge a TOPIC_REQUEST
pub struct RQ_Data{
    pub message_type: MessageType,
    pub status: TopicsResponse,
    pub topic_id: u64
}
impl RQ_Data {
    pub const fn new(status: TopicsResponse, topic_id: u64)-> RQ_Data {
        RQ_Data{message_type:MessageType::TOPIC_REQUEST_ACK, status, topic_id}
    }
}
*/
