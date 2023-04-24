use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::Rng;
use crate::enums::log_level::LogLevel::{Error, Info};
use crate::enums::object_identifier_type::ObjectIdentifierType;
use crate::libs::types::{ClientSender, ObjectId, PingId, ServerSocket};
use crate::{ISRUNNING, OBJECT_SUBSCRIBERS_REF, PINGS_REF};
use crate::enums::client_actions::ClientActions::UpdateServerLastRequest;
use crate::enums::log_source::LogSource::{ObjectHandler, PingSender};
use crate::libs::common::{log, now_ms};

/**===================================*
*                                     *
*           Objects methods           *
*                                     *
*=====================================*/

/**
 * This method ensure the given objectId
 * is a broker generated id and it still
 * exist.
 *
 * @param object_id: ObjectId, The object identifier
 *
 * @return bool
 */
pub async fn is_object_id_valid(
    object_id: ObjectId
) -> bool
{
    let existing_id = {
        OBJECT_SUBSCRIBERS_REF.read().await.get(&object_id).is_some()
    };

    return get_object_id_type(object_id) != ObjectIdentifierType::BROKER_GENERATED && existing_id;
}

/**
 * This method return the type of the
 * object id according to the two MSB
 *
 * @param id: ObjectId
 *
 * @return ObjectIdentifierType
 */
pub fn get_object_id_type(id: ObjectId) -> ObjectIdentifierType {
    // Use bitwise operator to get two MSB ->|XX______|
    let msb = (id >> 62) & 0b11;
    match msb {
        0b00 => {
            ObjectIdentifierType::USER_GENERATED
        }
        0b01 => {
            ObjectIdentifierType::BROKER_GENERATED
        }
        0b10 => {
            ObjectIdentifierType::TEMPORARY
        }
        _ => {
            ObjectIdentifierType::UNKNOWN
        }
    }
}

/**
 * This method generate a new object id and set
 * the two MSB according to the type needed.
 * /!\ Unknown type is not allow and will return 0.
 *
 * @param id_type: ObjectIdentifierType
 *
 * @return ObjectId or 0
 */
pub fn generate_object_id(id_type: ObjectIdentifierType) -> ObjectId {
    // 1 - Get the current time
    let now = match SystemTime::now().duration_since(UNIX_EPOCH){
        Ok(dur) => {
            // return the duration as millisecond
            dur.as_nanos() as u64
        }
        Err(err) => {
            log(Error, ObjectHandler, format!("Failed to get duration since UNIX_EPOCH in \"generate_object_id\". Error:\n{}", err));
            0 // return a default value to let the flow continuing
        }
    };



    let u64_with_msb_00 = now & 0x3FFFFFFFFFFFFFFF; // the mask allow to set two MSB to 00 to rewrite them after
    // 2 - set é MSB according to the type
    match id_type {
        ObjectIdentifierType::USER_GENERATED => {
            u64_with_msb_00 | 0x0000000000000000
        }
        ObjectIdentifierType::BROKER_GENERATED => {
            u64_with_msb_00 | 0x0100000000000000
        }
        ObjectIdentifierType::TEMPORARY => {
            u64_with_msb_00 | 0x1000000000000000
        }
        _ => 0
    }
}

/**
 * This method command asynchronously to a client to
 * update his last server request sent time to now.
 *
 * @param sender: ClientSender, The client channel to sent the command.
 */
pub async fn save_server_last_request_sent(
    sender: ClientSender
){
    // 1 - Create a new Command.
    let cmd = UpdateServerLastRequest {time: now_ms()};

    // 2 - Send it asynchronously through the client channel
    tokio::spawn(async move {
        match sender.send(cmd).await {
            Ok(_)=>{}
            Err(_)=> {
                return;
            }
        };
    });
}

/**
 * This methods return a unique id for a new ping reference
 *
 * @return PingId
 */
fn get_new_ping_id() -> PingId {
    // Return the XOR operation between the current time and a random PingId(unsigned int)
    return (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Failed to calculate duration since UNIX_EPOCH")
        .as_nanos() as PingId) ^ rand::thread_rng().gen::<PingId>();
}

/**
 * This method generate a new ping id and save the current time
 * as the "reference time" to compute the round trip later.
 *
 * @return PingId, The new ping id.
 */
pub async fn get_new_ping_reference() -> PingId {
    // 1 - Generate a new unique id
    let key = get_new_ping_id();

    // 2 - Get the current time as reference to compute the round trip later
    let time = now_ms();

    // 3 - Save it into the array
    PINGS_REF.lock().await.insert(key, time);
    log(Info, PingSender, format!("New ping reference created. Id : {}", key));

    // 4 - Return the id
    return key;
}

/**
 * This function sent the buffer to the address and
 * save the current time as the last server request
 * sent to this address.
 *
 * @param sender: ServerSocket, The socket used by the server to exchange data.
 * @param buffer: &[u8], The data that will be sent.
 * @param address: SocketAddr, The user address where the data will be sent.
 * @param client_sender: ClientSender, The channel used by the server to fire command to a client struct.
 *
 * @return std::io::Result<usize>, Number of bytes sent.
 */
pub async fn send_datagram(
    sender : ServerSocket,
    buffer: &[u8],
    address: SocketAddr,
    client_sender: ClientSender
) -> std::io::Result<usize>
{
    // 1 - Send the buffer to the address
    let result = sender.send_to(buffer, address).await;

    // 2 - Save the timestamp asynchronously
    let sender_ref = client_sender.clone();
    tokio::spawn(async move {
        save_server_last_request_sent(sender_ref).await;
    });
    // 3 - return the number of bytes sent
    return result;
}

/**
 * This method update the server status
 *
 * @param new_status: bool, The new status of the the server
 */
pub async fn update_server_status(new_status: bool){
    let mut status = ISRUNNING.write().await;
    *status= new_status;
    // write lock is dropped
}
