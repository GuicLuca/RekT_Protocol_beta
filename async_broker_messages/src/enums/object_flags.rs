/**
 * Object Flags are all possible action in a
 * OBJECT_REQUEST.
 */
#[derive(Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum ObjectFlags {
    CREATE,
    UPDATE,
    DELETE,
    SUBSCRIBE,
    UNSUBSCRIBE,
    UNKNOWN,
}


/**
 * This function convert an ObjectFlags to an u8
 *
 * @param value: ObjectFlags, The source to convert
 *
 * @return u8
 */
impl From<ObjectFlags> for u8 {
    fn from(value: ObjectFlags) -> Self {
        match value {
            ObjectFlags::CREATE => 0x01,
            ObjectFlags::UPDATE => 0x02,
            ObjectFlags::DELETE => 0x04,
            ObjectFlags::SUBSCRIBE => 0x08,
            ObjectFlags::UNSUBSCRIBE => 0x10,
            ObjectFlags::UNKNOWN => 0xAA,
        }
    }
}

/**
 * This function convert an u8 to a ObjectFlags
 *
 * @param value: u8, The source to convert
 *
 * @return ObjectFlags
 */
impl From<u8> for ObjectFlags {
    fn from(value: u8) -> Self {
        match value {
            0x01 => ObjectFlags::CREATE,
            0x02 => ObjectFlags::UPDATE,
            0x04 => ObjectFlags::DELETE,
            0x08 => ObjectFlags::SUBSCRIBE,
            0x10 => ObjectFlags::UNSUBSCRIBE,
            _ => ObjectFlags::UNKNOWN,
        }
    }
}