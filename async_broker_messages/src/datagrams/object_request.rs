#![allow(non_camel_case_types, unused)]
use std::collections::HashSet;
use std::mem::size_of;
use crate::enums::message_type::MessageType;
use crate::enums::object_flags::ObjectFlags;
use crate::libs::common::{get_bytes_from_slice, get_u16_at_pos, get_u64_at_pos};
use crate::libs::types::{ObjectId, Size};

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
        let size: u16 = (topics.len() * size_of::<u64>()) as u16; // x = size_of(topics)
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
        let mut bytes: Vec<u8> = Vec::with_capacity(12 + self.size as usize); // 3 = messageType + size
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

impl<'a> TryFrom<&'a [u8]> for RQ_ObjectRequest{
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 12 {
            return Err("Payload len is to short for a RQ_ObjectRequest.");
        }
        let size = get_u16_at_pos(buffer, 1)?;

        let mut topics: HashSet<u64>;
        if size != 0 {
            topics = get_bytes_from_slice(buffer, 12, (size as usize + 12))
                // Convert the bytes vector to a vector of topics id by grouping u8 into u64
                .chunks_exact(8)
                .map(|chunk| {
                    u64::from_le_bytes(chunk.try_into().unwrap())
                })
                .collect();
        }else{
            topics = HashSet::default();
        }

        let object_id = get_u64_at_pos(buffer,4)?;

        Ok(RQ_ObjectRequest {
            message_type: MessageType::from(buffer[0]),
            size,
            flags: ObjectFlags::from(buffer[3]),
            object_id,
            topics,
        })
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

impl<'a> TryFrom<&'a [u8]> for RQ_ObjectRequestCreate_ACK{
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 18 {
            return Err("Payload len is to short for a RQ_ObjectRequestCreate_ACK.");
        }
        let old_object_id = get_u64_at_pos(buffer,2)?;
        let final_object_id = get_u64_at_pos(buffer,10)?;

        Ok(RQ_ObjectRequestCreate_ACK {
            message_type: MessageType::from(buffer[0]),
            flags: buffer[1],
            old_object_id,
            final_object_id,
        })
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
        let mut bytes: Vec<u8> = Vec::with_capacity(10);
        bytes.push(u8::from(self.message_type));
        bytes.push(u8::from(self.flags));
        bytes.extend(self.object_id.to_le_bytes());

        return bytes;
    }
}

impl<'a> TryFrom<&'a [u8]> for RQ_ObjectRequestDefault_ACK{
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 10 {
            return Err("Payload len is to short for a RQ_ObjectRequestDefault_ACK.");
        }
        let object_id = get_u64_at_pos(buffer,2)?;

        Ok(RQ_ObjectRequestDefault_ACK {
            message_type: MessageType::from(buffer[0]),
            flags: buffer[1],
            object_id,
        })
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

impl<'a> TryFrom<&'a [u8]> for RQ_ObjectRequest_NACK{
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 12 {
            return Err("Payload len is to short for a RQ_ObjectRequest_NACK.");
        }
        let size = get_u16_at_pos(buffer,10)?;
        let object_id = get_u64_at_pos(buffer,2)?;

        Ok(RQ_ObjectRequest_NACK {
            message_type: MessageType::from(buffer[0]),
            flags: buffer[1],
            object_id,
            reason_size: size,
            reason: get_bytes_from_slice(buffer, 12, (size + 12) as usize)
        })
    }
}