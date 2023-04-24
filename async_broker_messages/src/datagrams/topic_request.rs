#![allow(non_camel_case_types, unused)]
use crate::enums::message_type::MessageType;
use crate::enums::topics_action::TopicsAction;
use crate::enums::topics_response::TopicsResponse;
use crate::libs::common::{get_bytes_from_slice, get_u16_at_pos, get_u64_at_pos};
use crate::types::Size;

//===== Sent to subscribe/unsubscribe to a topic
pub struct RQ_TopicRequest {
    pub message_type: MessageType, // 1 byte
    pub action: TopicsAction, // 1 byte
    pub topic_id: u64, // 8 bytes
}

//===== Sent to subscribe a topic
impl RQ_TopicRequest {
    pub fn new(action: TopicsAction, topic_id: u64) -> RQ_TopicRequest {
        RQ_TopicRequest { message_type: MessageType::TOPIC_REQUEST, action, topic_id }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(10);
        bytes.push(u8::from(self.message_type));
        bytes.push(u8::from(self.action));
        bytes.extend(self.topic_id.to_le_bytes());
        return bytes;
    }
}

impl<'a> TryFrom<&'a [u8]> for RQ_TopicRequest{
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 10 {
            return Err("Payload len is to short for a RQ_TopicRequest.");
        }
        let topic_id = get_u64_at_pos(buffer, 2)?;

        Ok(RQ_TopicRequest {
            message_type: MessageType::from(buffer[0]),
            action: TopicsAction::from(buffer[1]),
            topic_id
        })
    }
}

//===== Sent to acknowledge a TOPIC_REQUEST
pub struct RQ_TopicRequest_ACK {
    pub message_type: MessageType,
    pub status: TopicsResponse,
    pub topic_id: u64,
}

impl RQ_TopicRequest_ACK {
    pub const fn new(topic_id: u64, status: TopicsResponse) -> RQ_TopicRequest_ACK {
        RQ_TopicRequest_ACK { message_type: MessageType::TOPIC_REQUEST_ACK, status, topic_id }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(10);
        bytes.push(u8::from(self.message_type));
        bytes.push(u8::from(self.status));
        bytes.extend(self.topic_id.to_le_bytes());
        return bytes;
    }
}

impl<'a> TryFrom<&'a [u8]> for RQ_TopicRequest_ACK{
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 10 {
            return Err("Payload len is to short for a RQ_TopicRequest_Ack.");
        }
        let topic_id = get_u64_at_pos(buffer, 2)?;

        Ok(RQ_TopicRequest_ACK {
            message_type: MessageType::from(buffer[0]),
            status: TopicsResponse::from(buffer[1]),
            topic_id
        })
    }
}

pub struct RQ_TopicRequest_NACK {
    pub message_type: MessageType,
    pub status: TopicsResponse,
    pub size: Size,
    pub error_message: Vec<u8>
}

impl RQ_TopicRequest_NACK{

    pub fn new(status: TopicsResponse, error_message: &str) -> RQ_TopicRequest_NACK {
        let size = error_message.len() as u16; // string length + 1 for the action
        RQ_TopicRequest_NACK { message_type: MessageType::TOPIC_REQUEST_ACK, status, size, error_message: error_message.as_bytes().into()}
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(2 + self.error_message.len());
        bytes.push(u8::from(self.message_type));
        bytes.push(u8::from(self.status));
        bytes.extend(self.error_message.iter());

        return bytes;
    }
}

impl<'a> TryFrom<&'a [u8]> for RQ_TopicRequest_NACK{
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 10 {
            return Err("Payload len is to short for a RQ_TopicRequest.");
        }
        let size = get_u16_at_pos(buffer, 2)?;

        Ok(RQ_TopicRequest_NACK {
            message_type: MessageType::from(buffer[0]),
            status: TopicsResponse::from(buffer[1]),
            size,
            error_message: get_bytes_from_slice(buffer, 4, (4 + size) as usize)
        })
    }
}