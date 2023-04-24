use std::mem::size_of;

/**===================================*
*                                     *
*          Array manipulators         *
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
    let slice = get_bytes_from_slice(buffer, position, position+size_of::<u64>());
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
    let slice = get_bytes_from_slice(buffer, position, position+size_of::<u32>());
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
    let slice = get_bytes_from_slice(buffer, position, position+size_of::<u16>());
    if slice.len() != 2 {
        return Err("Slice len is invalid to convert it into an u16.")
    }
    Ok(u16::from_le_bytes(slice.try_into().unwrap()))
}