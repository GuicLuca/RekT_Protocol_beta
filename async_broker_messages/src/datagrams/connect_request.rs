#![allow(non_camel_case_types, unused)]
use crate::enums::connect_status::ConnectStatus;
use crate::enums::message_type::MessageType;
use crate::libs::common::{get_bytes_from_slice, get_u16_at_pos, get_u64_at_pos};
use crate::libs::types::Size;

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

impl<'a> TryFrom<&'a [u8]> for RQ_Connect{
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 1 {
            return Err("Payload len is to short for a RQ_Connect.");
        }
        Ok(RQ_Connect {
            message_type: MessageType::from(buffer[0]),
        })
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

impl<'a> TryFrom<&'a [u8]> for RQ_Connect_ACK_OK{
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 12 {
            return Err("Payload len is to short for a RQ_Connect_Ack_OK.");
        }
        let peer_id = get_u64_at_pos(buffer, 2)?;
        let heartbeat_period = get_u16_at_pos(buffer, 10)?;

        Ok(RQ_Connect_ACK_OK {
            message_type: MessageType::from(buffer[0]),
            status: ConnectStatus::from(buffer[1]),
            peer_id,
            heartbeat_period
        })
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
        let message_size = reason.len() as u16;
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

impl<'a> TryFrom<&'a [u8]> for RQ_Connect_ACK_ERROR{
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 4 {
            return Err("Payload len is to short for a RQ_Connect_Ack_Error.");
        }
        let size = get_u16_at_pos(buffer,1)?;

        Ok(RQ_Connect_ACK_ERROR {
            message_type: MessageType::from(buffer[0]),
            status: ConnectStatus::from(buffer[1]),
            message_size: size,
            reason: get_bytes_from_slice(buffer, 4, (4 + size) as usize),
        })
    }
}