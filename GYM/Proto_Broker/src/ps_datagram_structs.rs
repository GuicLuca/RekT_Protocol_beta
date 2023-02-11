use core::panicking::panic;
use hex::FromHex;

/** ==================================
 *
 *      ENUMERATIONS STRUCT &
 *         GLOBAL CLASSES
 *
 ** ================================*/

// Message type are used to translate
// request type to the corresponding code
#[derive(Copy, Clone)]
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
    DATA
}
impl From<u8> for MessageType{
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
            _ => panic!("Should not happen")
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
            _ => panic!("Should not happen")
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
}
impl From<ConnectStatus> for u8 {
    fn from(value: ConnectStatus) -> Self {
        match value {
            ConnectStatus::SUCCESS => 0x00,
            ConnectStatus::FAILURE => 0xFF,
            _ => panic!("Should not happend")
        }
    }
}
impl From<u8> for ConnectStatus {
    fn from(value: u8) -> Self {
        match value {
            0x00 => ConnectStatus::SUCCESS,
            0xFF => ConnectStatus::FAILURE,
            _ => panic!("Should not happend")
        }
    }
}



// End connexion reasons are used to
// detail the reason of the shutdown request.
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum EndConnexionReason {
    APPLICATION_SHUTDOWN,
    APPLICATION_ERROR,
}

impl From<u8> for EndConnexionReason {
    fn from(value: u8) -> Self {
        match value {
            0x00 => EndConnexionReason::APPLICATION_SHUTDOWN,
            0x01 => EndConnexionReason::APPLICATION_ERROR,
            _ => panic!("Should not happen")
        }
    }
}
impl From<EndConnexionReason> for u8 {
    fn from(value: EndConnexionReason) -> Self {
        match value {
             EndConnexionReason::APPLICATION_SHUTDOWN => 0x00,
             EndConnexionReason::APPLICATION_ERROR => 0x01,
            _ => panic!("Should not happen")
        }
    }
}

// End connexion reasons are used to
// detail the reason of the shutdown request.
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum StreamType {
    MANAGEMENT,
    RELIABLE_DATA,
    UNRELIABLE_DATA,
}
impl From<u8> for StreamType{
    fn from(value: u8) -> Self {
        match value {
            0x00 => StreamType::MANAGEMENT,
            0x01 => StreamType::RELIABLE_DATA,
            0x02 => StreamType::UNRELIABLE_DATA,
            _ => panic!("Should not happen")
        }
    }
}
impl From<StreamType> for u8{
    fn from(value: StreamType) -> Self {
        match value {
            StreamType::MANAGEMENT => 0x00,
            StreamType::RELIABLE_DATA => 0x01,
            StreamType::UNRELIABLE_DATA => 0x02,
            _ => panic!("Should not happen")
        }
    }
}

// Topics action are all actions that
// a peer can do in a TOPICS_REQUEST
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum TopicsAction {
    SUBSCRIBE = 0x00,
    UNSUBSCRIBE = 0xFF,
}
impl From<TopicsAction> for u8{
    fn from(value: TopicsAction) -> Self {
        match value {
            TopicsAction::SUBSCRIBE => 0x00,
            TopicsAction::UNSUBSCRIBE => 0xFF,
            _ => panic!("Should not happend")
        }
    }
}
impl From<u8> for TopicsAction{
    fn from(value: u8) -> Self {
        match value {
            0x00 => TopicsAction::SUBSCRIBE,
            0xFF => TopicsAction::UNSUBSCRIBE,
            _ => panic!("Should not happend")
        }
    }
}

// Topics response are all possible response
// type to a TOPICS_REQUEST
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum TopicsResponse {
    SUCCESS = 0x00,
    FAILURE = 0xF0,
}
impl From<TopicsResponse> for u8 {
    fn from(value: TopicsResponse) -> Self {
        match value {
            TopicsResponse::SUCCESS => 0x00,
            TopicsResponse::FAILURE => 0xF0,
            _ => panic!("Should not happend")
        }
    }
}
impl From<u8> for TopicsResponse {
    fn from(value: u8) -> Self {
        match value {
            0x00 => TopicsResponse::SUCCESS,
            0xF0 => TopicsResponse::FAILURE,
            _ => panic!("Should not happend")
        }
    }
}

pub struct Size{
    size:u16
}
impl Size {
    pub const fn new(size:u16) -> Size {
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
pub struct RQ_Connect{
    message_type: MessageType,
}
impl RQ_Connect {
    pub const fn new()-> RQ_Connect {
        RQ_Connect{message_type:MessageType::CONNECT}
    }

    pub fn as_bytes(&self)-> Vec<u8>
    {
        let bytes = [u8::from(self.message_type)].to_vec();

        return bytes;
    }
}

//===== Sent to acknowledge the connexion
pub struct RQ_Connect_ACK_OK{
    message_type: MessageType,
    status: ConnectStatus,
    peer_id:u64,
    heartbeat_period: u16,
}
impl RQ_Connect_ACK_OK {
    pub const fn new(peer_id:u64, heartbeat_period: u16)-> RQ_Connect_ACK_OK {
        RQ_Connect_ACK_OK{message_type:MessageType::CONNECT_ACK, status: ConnectStatus::SUCCESS, peer_id, heartbeat_period}
    }

    pub fn as_bytes(&self)-> Vec<u8>
    {
        let mut bytes = [u8::from(self.message_type)].to_vec();
        bytes.append(&mut [u8::from(self.status)].to_vec());
        bytes.append(&mut self.peer_id.to_le_bytes().to_vec());
        bytes.append(&mut self.heartbeat_period.to_le_bytes().to_vec());

        return bytes;
    }
}
pub struct RQ_Connect_ACK_ERROR{
    message_type: MessageType,
    status: ConnectStatus,
    message_size: Size,
    reason: Vec<u8>,
}
impl RQ_Connect_ACK_ERROR {
    pub const fn new(message_size:Size, reason: Vec<u8>)-> RQ_Connect_ACK_ERROR {
        RQ_Connect_ACK_ERROR{message_type:MessageType::CONNECT_ACK, status: ConnectStatus::FAILURE, message_size, reason}
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes = [u8::from(self.message_type)].to_vec();
        bytes.append(&mut [u8::from(self.status)].to_vec());
        bytes.append(&mut self.message_size.size.to_le_bytes().to_vec());
        bytes.append(&mut self.reason.clone());

        return bytes;
    }
}

//===== Sent to maintain the connexion
pub struct RQ_Heartbeat{
    message_type: MessageType,
}
impl RQ_Heartbeat {
    pub const fn new()-> RQ_Heartbeat {
        RQ_Heartbeat{message_type:MessageType::HEARTBEAT}
    }
}

//===== Sent to request a Heartbeat if a pear do not recive his
// normal heartbeat.
pub struct RQ_Heartbeat_Request{
    message_type: MessageType,
}
impl RQ_Heartbeat_Request {
    pub const fn new()-> RQ_Heartbeat_Request {
        RQ_Heartbeat_Request{message_type:MessageType::HEARTBEAT_REQUEST}
    }
}

//===== Sent to measure the latency between peer and broker
pub struct RQ_Ping{
    message_type: MessageType,
    ping_id:u8,
}
impl RQ_Ping {
    pub const fn new(ping_id:u8)-> RQ_Ping {
        RQ_Ping{message_type:MessageType::PING, ping_id}
    }
}

//===== Sent to answer a ping request.
pub struct RQ_Pong{
    message_type: MessageType,
    ping_id:u8,
}
impl RQ_Pong {
    pub const fn new(ping_id:u8)-> RQ_Ping {
        RQ_Ping{message_type:MessageType::PONG, ping_id}
    }
}

//===== Sent to close the connexion between peer and broker
pub struct RQ_Shutdown{
    message_type: MessageType,
    reason: EndConnexionReason,
}
impl RQ_Shutdown {
    pub const fn new(reason: EndConnexionReason)-> RQ_Shutdown {
        RQ_Shutdown{message_type:MessageType::SHUTDOWN, reason}
    }
}

//===== Sent to open a new stream
pub struct RQ_OpenStream{
    message_type: MessageType,
    stream_type: StreamType,
}
impl RQ_OpenStream {
    pub const fn new(stream_type: StreamType)-> RQ_OpenStream {
        RQ_OpenStream{message_type:MessageType::OPEN_STREAM, stream_type}
    }
}

//===== Sent to subscribe/unsubscribe to a topic
pub struct RQ_TopicRequest{
    message_type: MessageType,
    action: TopicsAction,
    topic_id: u64
}
impl RQ_TopicRequest {
    pub const fn new(action: TopicsAction, topic_id: u64)-> RQ_TopicRequest {
        RQ_TopicRequest{message_type:MessageType::TOPIC_REQUEST, action, topic_id}
    }
}

//===== Sent to acknowledge a TOPIC_REQUEST
pub struct RQ_TopicRequest_ACK{
    message_type: MessageType,
    status: TopicsResponse,
    topic_id: u64
}
impl RQ_TopicRequest_ACK {
    pub const fn new(status: TopicsResponse, topic_id: u64)-> RQ_TopicRequest_ACK {
        RQ_TopicRequest_ACK{message_type:MessageType::TOPIC_REQUEST_ACK, status, topic_id}
    }
}



/*
//===== Sent to acknowledge a TOPIC_REQUEST
pub struct RQ_ObjectRequest{
    message_type: MessageType,
    status: TopicsResponse,
    topic_id: u64
}
impl RQ_ObjectRequest {
    pub const fn new(status: TopicsResponse, topic_id: u64)-> RQ_ObjectRequest {
        RQ_ObjectRequest{message_type:MessageType::OBJECT_REQUEST, status, topic_id}
    }
}

//===== Sent to acknowledge a TOPIC_REQUEST
pub struct RQ_ObjectRequest_ACK{
    message_type: MessageType,
    status: TopicsResponse,
    topic_id: u64
}
impl RQ_ObjectRequest_ACK {
    pub const fn new(status: TopicsResponse, topic_id: u64)-> RQ_ObjectRequest_ACK {
        RQ_ObjectRequest_ACK{message_type:MessageType::OBJECT_REQUEST_ACK, status, topic_id}
    }
}

//===== Sent to acknowledge a TOPIC_REQUEST
pub struct RQ_Data{
    message_type: MessageType,
    status: TopicsResponse,
    topic_id: u64
}
impl RQ_Data {
    pub const fn new(status: TopicsResponse, topic_id: u64)-> RQ_Data {
        RQ_Data{message_type:MessageType::TOPIC_REQUEST_ACK, status, topic_id}
    }
}
*/
