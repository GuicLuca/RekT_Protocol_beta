/**
 * ConnectStatus are all possible status
 * in a CONNECT_ACK request.
 */
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum ConnectStatus {
    SUCCESS,
    FAILURE,
    UNKNOWN,
}

/**
 * This function convert a ConnectStatus to an u8
 *
 * @param value: ConnectStatus, The source to convert
 *
 * @return u8
 */
impl From<ConnectStatus> for u8 {
    fn from(value: ConnectStatus) -> Self {
        match value {
            ConnectStatus::SUCCESS => 0x00,
            ConnectStatus::FAILURE => 0xFF,
            ConnectStatus::UNKNOWN => 0xAA,
        }
    }
}

/**
 * This function convert a u8 to a ConnectStatus
 *
 * @param value: u8, The source to convert
 *
 * @return ConnectStatus
 */
impl From<u8> for ConnectStatus {
    fn from(value: u8) -> Self {
        match value {
            0x00 => ConnectStatus::SUCCESS,
            0xFF => ConnectStatus::FAILURE,
            _ => ConnectStatus::UNKNOWN
        }
    }
}