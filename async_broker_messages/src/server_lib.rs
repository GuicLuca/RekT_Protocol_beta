use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::time::{SystemTime, UNIX_EPOCH};
use rand::Rng;

use tokio::sync::{RwLockReadGuard};

use crate::client_lib::{now_ms};
use crate::client_lib::ClientActions::{UpdateServerLastRequest};
use crate::{CLIENTS_ADDRESSES_REF, CLIENTS_SENDERS_REF, CONFIG, PINGS_REF};
use crate::config::{LogLevel};
use crate::config::LogLevel::*;
use crate::datagram::*;
use crate::server_lib::LogSource::*;
use crate::types::{ClientId, ClientSender, ServerSocket, TopicId};

/**
 * LogSource are used to display prefix and filter log messages.
 */
pub enum LogSource {
    DatagramsHandler,
    PingSender,
    DataHandler,
    HeartbeatChecker,
    TopicHandler,
    ClientManager,
    Other,
}

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

    // For each case, check if the source is allowed by the CONFIGuration
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
    }
}


/**
 * This method return true if the client was already connected,
 *  it return the old client id too.
 *
 * @param ip: &IpAddr, ip address of the tested client
 * @param clients: MutexGuard<HashMap<ClientId, SocketAddr>>, Hashmap containing all clients address and clients id.
 *
 * @return (bool, ClientId)
 */
pub async fn already_connected<'a>(
    ip: &'a IpAddr,
    clients_addresses: RwLockReadGuard<'a, HashMap<ClientId,SocketAddr>>,
) -> (bool, ClientId) {
    // Use iterator for performance
    clients_addresses
        .iter()
        .find_map(|(key, value)| {
            if ip == &value.ip() {
                Some((true, *key))
            } else {
                None
            }
        })
        .unwrap_or((false, 0))
}

/**
 * This methods return a unique id for a new client.
 *
 * @param clients: MutexGuard<HashMap<ClientId, SocketAddr>>, Hashmap containing all clients address and clients id.
 *
 * @return ClientId
 */
pub fn get_new_id() -> ClientId {
    // Return the XOR operation between the current time and a random u64
    return (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Failed to calculate duration since UNIX_EPOCH")
        .as_nanos() as ClientId) ^ rand::thread_rng().gen::<ClientId>();
}


/**
 * This methods return the id of the given client and None if the
 * source is not found.
 *
 * @param src: SocketAddr, The searched source.
 * @param clients MutexGuard<HashMap<u64, SocketAddr>> : Hashmap containing all clients address and clients id.
 *
 * @return Option<ClientId>
 */
pub async fn get_client_id(
    src: SocketAddr,
) -> Option<ClientId> {
    // Use the iterator for performance.
    CLIENTS_ADDRESSES_REF.read().await
        .iter()
        .find_map(|(&key, &addr)| if addr == src { Some(key) } else { None })
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
 * @param pings MutexGuard<HashMap<u8, u128>> : Hashmap containing all ping id and their time reference.
 *
 * @return u8
 */
fn get_new_ping_id() -> u8 {
    // Return the XOR operation between the current time and a random u8
    return (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Failed to calculate duration since UNIX_EPOCH")
        .as_nanos() as u8) ^ rand::thread_rng().gen::<u8>();
}

/**
 * This method generate a new ping id and save the current time
 * as the "reference time" to compute the round trip later.
 *
 * @param pings: PingsHashMap, The HashMap that contain every ping reference.
 *
 * @return u8, The new ping id.
 */
pub async fn get_new_ping_reference() -> u8 {
    // 1 - Generate a new unique id
    let key = get_new_ping_id();

    // 2 - Get the current time as reference to compute the round trip later
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis(); // Current time in ms

    // 3 - Save it into the array
    PINGS_REF.lock().await.insert(key, time);
    log(Info, PingSender, format!("New ping reference created. Id : {}", key));

    // 4 - Return the id
    return key;
}


/**
 * This function return true if the client is currently online.
 *
 * @param client_id: ClientId,  The checked client identifier
 * @param clients ClientsHashMap<ClientSender>: The hashmap of all connected clients
 *
 * @return bool
 */
pub async fn is_online(
    client_id: ClientId,
) -> bool
{
    let clients_read = CLIENTS_SENDERS_REF.read().await;
    return clients_read.contains_key(&client_id);
}

const FNV_PRIME: u64 = 1099511628211;
const FNV_OFFSET: u64 = 14695981039346656037;
/**
 * This method hash the passed string and return
 * the result as a topic id.
 *
 * @param s &str: the string to hash
 *
 * @return TopicId: the hash of the string
 */
pub fn custom_string_hash(
    string: &str
) -> TopicId {
    // 1 - Init the hash with the offset
    let mut hash = FNV_OFFSET;
    // 2 - Loop through each bytes and do a XOR
    // operation with the current hash and the bit value.
    // Finally multiply the result by a constant
    string.bytes().into_iter().for_each(|b| {
        hash ^= b as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    });
    // 3 -  return the hashed value
    hash
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