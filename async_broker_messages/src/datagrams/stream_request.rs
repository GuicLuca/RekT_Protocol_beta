#![allow(non_camel_case_types, unused)]
use crate::enums::message_type::MessageType;
use crate::enums::stream_type::StreamType;

//===== Sent to open a new stream
pub struct RQ_OpenStream {
    pub message_type: MessageType,
    pub stream_type: StreamType,
}

impl RQ_OpenStream {
    pub const fn new(stream_type: StreamType) -> RQ_OpenStream {
        RQ_OpenStream { message_type: MessageType::OPEN_STREAM, stream_type }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(2);
        bytes.push(u8::from(self.message_type));
        bytes.push(u8::from(self.stream_type));
        return bytes;
    }
}

impl<'a> TryFrom<&'a [u8]> for RQ_OpenStream{
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 2 {
            return Err("Payload len is to short for a RQ_OpenStream.");
        }

        Ok(RQ_OpenStream {
            message_type: MessageType::from(buffer[0]),
            stream_type: StreamType::from(buffer[1])
        })
    }
}