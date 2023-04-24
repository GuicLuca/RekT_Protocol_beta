#![allow(non_camel_case_types, unused)]
/**
 * End connexion reasons are used to
 * detail the reason of the shutdown request.
 */
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum EndConnexionReason {
    SHUTDOWN,
    ERROR,
    SYNC_ERROR,
    UNKNOWN,
}

/**
 * This function convert a u8 to an EndConnexionReason
 *
 * @param value: u8, The source to convert
 *
 * @return EndConnexionReason
 */
impl From<u8> for EndConnexionReason {
    fn from(value: u8) -> Self {
        match value {
            0x00 => EndConnexionReason::SHUTDOWN,
            0x01 => EndConnexionReason::ERROR,
            0x02 => EndConnexionReason::SYNC_ERROR,
            _ => EndConnexionReason::UNKNOWN,
        }
    }
}

/**
 * This function convert an EndConnexionReason to an u8
 *
 * @param value: EndConnexionReason, The source to convert
 *
 * @return u8
 */
impl From<EndConnexionReason> for u8 {
    fn from(value: EndConnexionReason) -> Self {
        match value {
            EndConnexionReason::SHUTDOWN => 0x00,
            EndConnexionReason::ERROR => 0x01,
            EndConnexionReason::SYNC_ERROR => 0x02,
            EndConnexionReason::UNKNOWN => 0xAA,
        }
    }
}