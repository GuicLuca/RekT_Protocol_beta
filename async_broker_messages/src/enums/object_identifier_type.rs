#![allow(non_camel_case_types, unused)]
/**
 * Object identifier types are all possible source of identifier.
 * It Represent the twoMSB of the Object identifier : XX
 * 0--2------------64
 * |XX| IDENTIFIER |
 */
#[derive(Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum ObjectIdentifierType {
    USER_GENERATED,
    BROKER_GENERATED,
    TEMPORARY,
    UNKNOWN
}


/**
 * This function convert an ObjectIdentifierType to an u8
 *
 * @param value: ObjectIdentifierType, The source to convert
 *
 * @return u8
 */
impl From<ObjectIdentifierType> for u8 {
    fn from(value: ObjectIdentifierType) -> Self {
        match value {
            ObjectIdentifierType::USER_GENERATED => 0x00,
            ObjectIdentifierType::BROKER_GENERATED => 0x01,
            ObjectIdentifierType::TEMPORARY => 0x10,
            ObjectIdentifierType::UNKNOWN => 0xAA,
        }
    }
}

/**
 * This function convert an u8 to a ObjectIdentifierType
 *
 * @param value: u8, The source to convert
 *
 * @return ObjectIdentifierType
 */
impl From<u8> for ObjectIdentifierType {
    fn from(value: u8) -> Self {
        match value {
            0x00 => ObjectIdentifierType::USER_GENERATED,
            0x01 => ObjectIdentifierType::BROKER_GENERATED,
            0x10 => ObjectIdentifierType::TEMPORARY,
            _ => ObjectIdentifierType::UNKNOWN,
        }
    }
}