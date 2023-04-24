/**
 * Topics action are all actions that
 * a peer can do in a TOPICS_REQUEST
 */
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum TopicsAction {
    SUBSCRIBE,
    UNSUBSCRIBE,
    UNKNOWN,
}

/**
 * This function convert a TopicsAction to an u8
 *
 * @param value: TopicsActions, The source to convert
 *
 * @return u8
 */
impl From<TopicsAction> for u8 {
    fn from(value: TopicsAction) -> Self {
        match value {
            TopicsAction::SUBSCRIBE => 0x00,
            TopicsAction::UNSUBSCRIBE => 0xFF,
            TopicsAction::UNKNOWN => 0xAA,
        }
    }
}

/**
 * This function convert an u8 to a TopicsActions
 *
 * @param value: u8, The source to convert
 *
 * @return TopicsActions
 */
impl From<u8> for TopicsAction {
    fn from(value: u8) -> Self {
        match value {
            0x00 => TopicsAction::SUBSCRIBE,
            0xFF => TopicsAction::UNSUBSCRIBE,
            _ => TopicsAction::UNKNOWN
        }
    }
}