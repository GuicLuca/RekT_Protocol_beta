#![allow(non_camel_case_types, unused)]

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