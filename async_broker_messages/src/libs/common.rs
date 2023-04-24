use std::collections::HashSet;
use std::mem::size_of;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::CONFIG;
use crate::enums::log_level::{display_loglevel, LogLevel};
use crate::enums::log_level::LogLevel::Error;
use crate::enums::log_source::LogSource;
use crate::enums::log_source::LogSource::*;
use crate::libs::types::TopicId;

/**===================================*
*                                     *
*     Array/vec/set manipulators      *
*                                     *
*=====================================*/

/**
 * This functions return a bytes slice according to
 * the given bounds. FROM and TO are include in the returned slice.
 *
 * @param buffer: &[u8], the original array,
 * @param from: usize, first bound,
 * @param to: usize, last bound,
 *
 * @return Vec<u8>, the slice requested
 */
pub fn get_bytes_from_slice(
    buffer: &[u8],
    from: usize,
    to: usize
) -> Vec<u8> {
    // 1 - check bound validity
    match () {
        _ if to < from => panic!("from is greater than to"),
        _ if to >= buffer.len() => panic!("to is greater than the last index"),
        _ => (),
    }

    // 2 - return the correct slice
    buffer[from..to+1].into()
}


/**
 * This method is an helper to find an u64 at position
 * in a buffer of u8
 *
 * @param buffer: &[u8], the source of the u64
 * @param position: usize, the position of the first byte of the u64
 *
 * @return u64
 */
pub fn get_u64_at_pos(buffer: &[u8], position: usize) -> Result<u64, &str>
{
    let slice = get_bytes_from_slice(buffer, position, position+size_of::<u64>()-1);
    if slice.len() != 8 {
        return Err("Slice len is invalid to convert it into an u64.")
    }
    Ok(u64::from_le_bytes(slice.try_into().unwrap()))
}
/**
 * This method is an helper to find an u32 at position
 * in a buffer of u8
 *
 * @param buffer: &[u8], the source of the u32
 * @param position: usize, the position of the first byte of the u32
 *
 * @return u32
 */
pub fn get_u32_at_pos(buffer: &[u8], position: usize) -> Result<u32, &str>
{
    let slice = get_bytes_from_slice(buffer, position, position+size_of::<u32>()-1);
    if slice.len() != 4 {
        return Err("Slice len is invalid to convert it into an u32.")
    }
    Ok(u32::from_le_bytes(slice.try_into().unwrap()))
}
/**
 * This method is an helper to find an u16 at position
 * in a buffer of u8
 *
 * @param buffer: &[u8], the source of the u16
 * @param position: usize, the position of the first byte of the u16
 *
 * @return u16
 */
pub fn get_u16_at_pos(buffer: &[u8], position: usize) -> Result<u16, &str>
{
    let slice = get_bytes_from_slice(buffer, position, position+size_of::<u16>()-1);
    if slice.len() != 2 {
        return Err("Slice len is invalid to convert it into an u16.")
    }
    Ok(u16::from_le_bytes(slice.try_into().unwrap()))
}


/**
 * This method return a tuple off two vec containing
 * added and removed values.
 *
 * @param new_set: &HashSet<TopicId>, The new set containing incoming values
 * @param current_set: &HashSet<TopicId>, the current set containing actual values
 *
 * @return added_values, removed_values: (Vec<TopicId>, Vec<TopicId>): two vectors containing differences from the original set
 */
pub fn diff_hashsets(new_set: &HashSet<TopicId>, current_set: &HashSet<TopicId>) -> (Vec<TopicId>, Vec<TopicId>) {
    let added_values = new_set.difference(current_set).cloned().collect();
    let removed_values = current_set.difference(new_set).cloned().collect();
    (added_values, removed_values)
}

/**
 * This method return the u8 image of the
 * given bitfields. The bitfield must be in LE endian
 *
 * @param bitfields: Vec<u8>
 *
 * @return u8
 */
pub fn vec_to_u8(bitfield: Vec<u8>) -> u8{
    (bitfield[0] << 7) | (bitfield[1] << 6) | (bitfield[2] << 5) | (bitfield[3] << 4) | (bitfield[4] << 3) | (bitfield[5] << 2) | (bitfield[6] << 1) | (bitfield[7] << 0)
}

/**===================================*
*                                     *
*            time methods             *
*                                     *
*=====================================*/

/**
 * Return the local time since the UNIX_EPOCH in ms
 *
 * @return u128, the current time in ms
 */
pub fn now_ms() -> u128
{
    match SystemTime::now().duration_since(UNIX_EPOCH){
        Ok(dur) => {
            // return the duration as millisecond
            dur.as_millis()
        }
        Err(err) => {
            log(Error, Other, format!("Failed to get duration since UNIX_EPOCH in now_ms. Error:\n{}", err));
            0 // return a default value to let the flow continuing
        }
    }
}


/**===================================*
*                                     *
*         logging methods             *
*                                     *
*=====================================*/

/**
 * This method print messages with source prefix.
 * Before printing anything, it check if the log level
 * is allowed by the config.
 *
 * @param log_level: LogLevel, The level of importance of the message.
 * @param log_source: LogSource, The source of the message.
 * @param message: String, The message to print.
 */
pub fn log(
    log_level: LogLevel,
    log_source: LogSource,
    message: String
) {
    // If log level is under CONFIG log level do not show the message
    if log_level < CONFIG.debug_level { return; }

    // For each case, check if the source is allowed by the configuration
    // and then, print the message.
    match log_source {
        DatagramsHandler => {
            if !CONFIG.debug_datagram_handler { return; }
            println!("[Server - DatagramHandler] {}: {}", display_loglevel(log_level), message);
        }
        PingSender => {
            if !CONFIG.debug_ping_sender { return; }
            println!("[Server - PingSender] {}: {}", display_loglevel(log_level), message);
        }
        DataHandler => {
            if !CONFIG.debug_data_handler { return; }
            println!("[Server - DataHandler] {}: {}", display_loglevel(log_level), message);
        }
        HeartbeatChecker => {
            if !CONFIG.debug_heartbeat_checker { return; }
            println!("[Server - HeartbeatChecker] {}: {}", display_loglevel(log_level), message);
        }
        TopicHandler => {
            if !CONFIG.debug_topic_handler { return; }
            println!("[Server - TopicHandler] {}: {}", display_loglevel(log_level), message);
        }
        Other => {
            println!("[Server] {}", message);
        }
        ClientManager => {
            if !CONFIG.debug_topic_handler { return; }
            println!("[Server - ClientManger] {}: {}", display_loglevel(log_level), message);
        }
        ObjectHandler => {
            if !CONFIG.debug_object_handler { return; }
            println!("[Server - ObjectHandler] {}: {}", display_loglevel(log_level), message);
        }
    }
}