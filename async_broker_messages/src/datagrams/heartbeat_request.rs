#![allow(non_camel_case_types, unused)]
use crate::enums::message_type::MessageType;

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

impl<'a> TryFrom<&'a [u8]> for RQ_Heartbeat{
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 1 {
            return Err("Payload len is to short for a RQ_Heartbeat.");
        }

        Ok(RQ_Heartbeat {
            message_type: MessageType::from(buffer[0]),
        })
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
impl<'a> TryFrom<&'a [u8]> for RQ_Heartbeat_Request{
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 1 {
            return Err("Payload len is to short for a RQ_Heartbeat_Request.");
        }

        Ok(RQ_Heartbeat_Request {
            message_type: MessageType::from(buffer[0]),
        })
    }
}