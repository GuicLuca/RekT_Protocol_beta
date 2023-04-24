#![allow(non_camel_case_types, unused)]
use crate::enums::message_type::MessageType;
use crate::types::PingId;

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

impl<'a> TryFrom<&'a [u8]> for RQ_Ping{
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 2 {
            return Err("Payload len is to short for a RQ_Ping.");
        }

        Ok(RQ_Ping {
            message_type: MessageType::from(buffer[0]),
            ping_id: buffer[1]
        })
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

impl<'a> TryFrom<&'a [u8]> for RQ_Pong{
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 2 {
            return Err("Payload len is to short for a RQ_Pong.");
        }

        Ok(RQ_Pong {
            message_type: MessageType::from(buffer[0]),
            ping_id: buffer[1]
        })
    }
}