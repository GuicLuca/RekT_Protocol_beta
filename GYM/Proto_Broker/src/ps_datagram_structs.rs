/** ==================================
 *
 *      ENUMERATIONS STRUCT &
 *         GLOBAL CLASSES
 *
 ** ================================*/

// Message type are used to translate
// request type to the corresponding code
pub enum MessageType {
    CONNECT = 0xF0,
    CONNECT_ACK = 0xF1,
    OPEN_STREAM = 0xF2,
    SHUTDOWN = 0xFF,
    HEARTBEAT = 0x01,
    HEARTBEAT_REQUEST = 0x41,
    PING = 0x02,
    PONG = 0x42,
    TOPIC_REQUEST = 0x07,
    TOPIC_REQUEST_ACK = 0x47,
    OBJECT_REQUEST = 0x08,
    OBJECT_REQUEST_ACK = 0x48,
    DATA = 0x05
}

// End connexion reasons are used to
// detail the reason of the shutdown request.
pub enum EndConnexionReason {
    APPLICATION_SHUTDOWN = 0x00,
    APPLICATION_ERROR = 0x01,
}

pub struct Size{
    size:i16
}
impl Size {
    pub const fn new(size:i16) -> Size {
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

// Sent to connect to a peer to the server
pub struct RQ_Connect{
    message_type: MessageType,
}
impl RQ_Connect {
    pub const fn new()-> RQ_Connect {
        RQ_Connect{message_type:MessageType::CONNECT}
    }
}



// Sent to acknowledge the connexion
pub struct RQ_Connect_ACK_OK{
    message_type: MessageType,
    status: i8,
    peer_id:i64,
    heartbeat_period: i16,
}
impl RQ_Connect_ACK_OK {
    pub const fn new(peer_id:i64, heartbeat_period: i16)-> RQ_Connect_ACK_OK {
        RQ_Connect_ACK_OK{message_type:MessageType::CONNECT_ACK, status: 0x00, peer_id, heartbeat_period}
    }
}
pub struct RQ_Connect_ACK_ERROR{
    message_type: MessageType,
    status: i8,
    message_size: Size,
    reason: Vec<u8>,
}
impl RQ_Connect_ACK_ERROR {
    pub const fn new(message_size:Size, reason: Vec<u8>)-> RQ_Connect_ACK_ERROR {
        RQ_Connect_ACK_ERROR{message_type:MessageType::CONNECT_ACK, status: 0xFF, message_size, reason}
    }
}

// Sent to maintain the connexion
pub struct RQ_Heartbeat{
    message_type: MessageType,
}
impl RQ_Heartbeat {
    pub const fn new()-> RQ_Heartbeat {
        RQ_Heartbeat{message_type:MessageType::HEARTBEAT}
    }
}

// Sent to request a Heartbeat if a pear do not recive his
// normal heartbeat.
pub struct RQ_Heartbeat_Request{
    message_type: MessageType,
}
impl RQ_Heartbeat_Request {
    pub const fn new()-> RQ_Heartbeat_Request {
        RQ_Heartbeat_Request{message_type:MessageType::HEARTBEAT_REQUEST}
    }
}

// Sent to measure the latency between peer and broker
pub struct RQ_Ping{
    message_type: MessageType,
    ping_id:i8,
}
impl RQ_Ping {
    pub const fn new(ping_id:i8)-> RQ_Ping {
        RQ_Ping{message_type:MessageType::PING, ping_id}
    }
}

// Sent to answer a ping request.
pub struct RQ_Pong{
    message_type: MessageType,
    ping_id:i8,
}
impl RQ_Pong {
    pub const fn new(ping_id:i8)-> RQ_Ping {
        RQ_Ping{message_type:MessageType::PONG, ping_id}
    }
}

// Sent to close the connexion between peer and broker
pub struct RQ_Shutdown{
    message_type: MessageType,
    reason: EndConnexionReason,
}
impl RQ_Shutdown {
    pub const fn new(reason: EndConnexionReason)-> RQ_Shutdown {
        RQ_Shutdown{message_type:MessageType::SHUTDOWN, reason}
    }
}

// Sent to open a new stream