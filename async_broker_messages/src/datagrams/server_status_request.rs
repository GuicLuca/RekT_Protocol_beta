#![allow(non_camel_case_types, unused)]
use crate::enums::message_type::MessageType;
use crate::libs::common::get_u64_at_pos;
use crate::libs::types::ClientId;

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
            message_type: MessageType::from(buffer[0]),
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

impl<'a> TryFrom<&'a [u8]> for RQ_ServerStatus_ACK {
    type Error = &'a str;

    fn try_from(buffer: &'a[u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 10 {
            return Err("Payload len is to short for a RQ_ServerStatus_Ack.");
        }
        let connected_client = get_u64_at_pos(buffer,2)?;
        Ok(RQ_ServerStatus_ACK {
            message_type: MessageType::from(buffer[0]),
            status: buffer[1] != 0,
            connected_client,
        })
    }
}