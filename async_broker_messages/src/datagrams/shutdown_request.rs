#![allow(non_camel_case_types, unused)]
use crate::enums::end_connexion_reason::EndConnexionReason;
use crate::enums::message_type::MessageType;

//===== Sent to close the connexion between peer and broker
pub struct RQ_Shutdown {
    pub message_type: MessageType,
    pub reason: EndConnexionReason,
}

impl RQ_Shutdown {
    pub const fn new(reason: EndConnexionReason) -> RQ_Shutdown {
        RQ_Shutdown { message_type: MessageType::SHUTDOWN, reason }
    }

    pub fn as_bytes(&self) -> Vec<u8>
    {
        let mut bytes: Vec<u8> = Vec::with_capacity(2);
        bytes.push(u8::from(self.message_type));
        bytes.push(u8::from(self.reason));
        return bytes;
    }
}
impl<'a> TryFrom<&'a [u8]> for RQ_Shutdown{
    type Error = &'a str;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        if buffer.len() < 2 {
            return Err("Payload len is to short for a RQ_Shutdown.");
        }

        Ok(RQ_Shutdown {
            message_type: MessageType::from(buffer[0]),
            reason: EndConnexionReason::from(buffer[1])
        })
    }
}