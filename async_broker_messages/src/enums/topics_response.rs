#![allow(non_camel_case_types, unused)]
/**
 * Topics response are all possible responses
 * type to a TOPICS_REQUEST
 */
#[derive(Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum TopicsResponse {
    SUCCESS_SUB,
    FAILURE_SUB,
    SUCCESS_USUB,
    FAILURE_USUB,
    UNKNOWN,
}

/**
 * This function convert an TopicsResponse to an u8
 *
 * @param value: TopicsResponse, The source to convert
 *
 * @return u8
 */
impl From<TopicsResponse> for u8 {
    fn from(value: TopicsResponse) -> Self {
        match value {
            TopicsResponse::SUCCESS_SUB => 0x00,
            TopicsResponse::SUCCESS_USUB => 0x0F,
            TopicsResponse::FAILURE_SUB => 0xF0,
            TopicsResponse::FAILURE_USUB => 0xFF,
            TopicsResponse::UNKNOWN => 0xAA,
        }
    }
}

/**
 * This function convert an u8 to a TopicsResponse
 *
 * @param value: u8, The source to convert
 *
 * @return TopicsResponse
 */
impl From<u8> for TopicsResponse {
    fn from(value: u8) -> Self {
        match value {
            0x00 => TopicsResponse::SUCCESS_SUB,
            0x0F => TopicsResponse::SUCCESS_USUB,
            0xF0 => TopicsResponse::FAILURE_SUB,
            0xFF => TopicsResponse::FAILURE_USUB,
            _ => TopicsResponse::UNKNOWN
        }
    }
}