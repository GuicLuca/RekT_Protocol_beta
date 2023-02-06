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

pub struct RQ_Connect_ACK{
    message_type: MessageType,
    heartbeat_period: i16,
}
impl RQ_Connect_ACK {
    pub const fn new(heartbeat_period:i16)-> RQ_Connect_ACK {
        RQ_Connect_ACK{message_type:MessageType::CONNECT_ACK, heartbeat_period}
    }
}

pub struct RQ_Heartbeat{
    message_type: MessageType,
}
impl RQ_Heartbeat {
    pub const fn new()-> RQ_Heartbeat {
        RQ_Heartbeat{message_type:MessageType::HEARTBEAT}
    }
}

pub struct RQ_Heartbeat_Request{
    message_type: MessageType,
}
impl RQ_Heartbeat_Request {
    pub const fn new()-> RQ_Heartbeat_Request {
        RQ_Heartbeat_Request{message_type:MessageType::HEARTBEAT_REQUEST}
    }
}

pub struct RQ_Ping{
    message_type: MessageType,
    ping_id:i8,
}
impl RQ_Ping {
    pub const fn new(ping_id:i8)-> RQ_Ping {
        RQ_Ping{message_type:MessageType::PING, ping_id}
    }
}

pub struct RQ_Pong{
    message_type: MessageType,
    ping_id:i8,
}
impl RQ_Pong {
    pub const fn new(ping_id:i8)-> RQ_Ping {
        RQ_Ping{message_type:MessageType::PONG, ping_id}
    }
}