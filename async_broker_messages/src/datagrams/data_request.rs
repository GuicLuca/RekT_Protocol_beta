#![allow(non_camel_case_types, unused)]
use crate::enums::message_type::MessageType;
use crate::libs::common::{get_u16_at_pos, get_u32_at_pos, get_u64_at_pos};
use crate::types::Size;

pub struct RQ_Data{
    pub message_type: MessageType, // 1 byte
    pub size: Size, // 2 bytes (u16)
    pub sequence_number: u32, // 4 bytes (u32)
    pub topic_id: u64, // 8 bytes (u64)
    pub data: Vec<u8> // size bytes
}
impl RQ_Data {

    pub fn new(sequence_number: u32, topic_id: u64, payload: Vec<u8>)-> RQ_Data {
        let size = payload.len() as u16;
        RQ_Data { message_type: MessageType::DATA, size, sequence_number, topic_id, data: payload }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(15 + self.data.len());
        bytes.push(u8::from(self.message_type));
        bytes.extend(self.size.to_le_bytes());
        bytes.extend(self.sequence_number.to_le_bytes());
        bytes.extend(self.topic_id.to_le_bytes());
        bytes.extend(self.data.iter());
        return bytes;
    }
}

impl<'a> TryFrom<&'a [u8]> for RQ_Data{
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 15 {
            return Err("Payload len is to short for a RQ_Data.");
        }
        let size = get_u16_at_pos(buffer, 1)?;
        let sequence_number = get_u32_at_pos(buffer, 3)?;
        let topic_id = get_u64_at_pos(buffer, 7)?;

        Ok(RQ_Data {
            message_type: MessageType::DATA,
            size,
            sequence_number,
            topic_id,
            data: buffer[15..15+size as usize].into(),
        })
    }
}