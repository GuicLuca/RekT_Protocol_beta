use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::sync::{RwLockReadGuard};

use crate::client_lib::{now_ms};
use crate::client_lib::ClientActions::{UpdateServerLastRequest};
use crate::config::{Config, LogLevel};
use crate::config::LogLevel::*;
use crate::datagram::*;
use crate::server_lib::LogSource::*;
use crate::types::{ClientId, ClientSender, ClientsHashMap, PingsHashMap, ServerSocket, TopicId};

pub enum LogSource {
    DatagramsHandler,
    PingSender,
    DataHandler,
    HeartbeatChecker,
    TopicHandler,
    ClientManager,
    Other,
}

pub fn log(
    log_level: LogLevel,
    log_source: LogSource,
    message: String,
    config: Arc<Config>,
) {
    // If log level is under config log level do not show the message
    if log_level < config.debug_level { return; }

    match log_source {
        DatagramsHandler => {
            if !config.debug_datagram_handler { return; }
            println!("[Server - DatagramHandler] {}: {}", display_loglevel(log_level), message);
        }
        PingSender => {
            if !config.debug_ping_sender { return; }
            println!("[Server - PingSender] {}: {}", display_loglevel(log_level), message);
        }
        DataHandler => {
            if !config.debug_data_handler { return; }
            println!("[Server - DataHandler] {}: {}", display_loglevel(log_level), message);
        }
        HeartbeatChecker => {
            if !config.debug_heartbeat_checker { return; }
            println!("[Server - HeartbeatChecker] {}: {}", display_loglevel(log_level), message);
        }
        TopicHandler => {
            if !config.debug_topic_handler { return; }
            println!("[Server - TopicHandler] {}: {}", display_loglevel(log_level), message);
        }
        Other => {
            println!("[Server] {}", message);
        }
        ClientManager => {
            if !config.debug_topic_handler { return; }
            println!("[Server - ClientManger] {}: {}", display_loglevel(log_level), message);
        }
    }
}


/**
This method return true if the client was already connected, it return the old
client id too.
@param ip &IpAddr : ip address of the tested client
@param clients MutexGuard<HashMap<u64, SocketAddr>> : Hashmap containing all clients address and clients id.

@return (bool, u64)
 */
pub async fn already_connected<'a>(
    ip: &'a IpAddr,
    clients_addresses: RwLockReadGuard<'a, HashMap<u64,SocketAddr>>,
) -> (bool, ClientId) {
    for (key, value) in clients_addresses.iter() {
        if ip == &value.ip() {
            return (true, *key);
        }
    }
    return (false, 0);
}

/**
This methods return a uniq id for a new client
@param clients MutexGuard<HashMap<u64, SocketAddr>> : Hashmap containing all clients address and clients id.

@return u64
 */
pub fn get_new_id() -> ClientId {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos();

    let xor_key: u64 = rand::random();
    // return the XOR operation of the current time and the random number
    return (timestamp as u64) ^ xor_key;
}

/**
This methods return a uniq id for a new ping reference
@param pings MutexGuard<HashMap<u8, u128>> : Hashmap containing all ping id and the time reference.

@return u8
 */
fn get_new_ping_id() -> u8 {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos();

    let xor_key: u8 = rand::random();
    // return the XOR operation of the current time and the random number
    return (timestamp as u8) ^ xor_key;
}

/**
This methods return the id of the given client
@param src &SocketAddr : The tested client
@param clients MutexGuard<HashMap<u64, SocketAddr>> : Hashmap containing all clients address and clients id.

@return Option<u64>
 */
pub async fn get_client_id(
    src: &SocketAddr,
    clients_addresses: HashMap<u64, SocketAddr>,
) -> Option<ClientId> {
    for (key, addr) in clients_addresses.iter() {
        if addr == src {
            return Some(*key);
        }
    }
    None
}


pub async fn save_server_last_request_sent(
    sender: ClientSender
){
    let cmd = UpdateServerLastRequest {time: now_ms()};
    tokio::spawn(async move {
        match sender.send(cmd).await {
            Ok(_)=>{}
            Err(_)=> {
                return;
            }
        };
    });
}


pub async fn get_new_ping_reference(pings: PingsHashMap, config: Arc<Config>) -> u8 {
    let key = get_new_ping_id();

    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis(); // Current time in ms

    pings.lock().await.insert(key, time);
    log(Info, PingSender, format!("New ping reference created. Id : {}", key), config.clone());
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
    clients: ClientsHashMap<ClientSender>,
) -> bool
{
    let clients_read = clients.read().await;
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