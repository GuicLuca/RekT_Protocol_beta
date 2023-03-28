use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::net::UdpSocket;
use tokio::sync::{mpsc, Mutex, RwLock, RwLockReadGuard};
use tokio::sync::mpsc::Sender;

use crate::client::Client;
use crate::client_lib::{ClientActions, now_ms};
use crate::client_lib::ClientActions::{StartManagers, UpdateServerLastRequest};
use crate::config::{Config, LogLevel};
use crate::config::LogLevel::*;
use crate::datagram::*;
use crate::server_lib::LogSource::*;
use crate::topic::Topic;

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
) -> (bool, u64) {
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
pub fn get_new_id() -> u64 {
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
) -> Option<u64> {
    for (key, addr) in clients_addresses.iter() {
        if addr == src {
            return Some(*key);
        }
    }
    None
}


pub async fn save_server_last_request_sent(
    sender: Sender<ClientActions>,
    client_id: u64,
){
    let cmd = UpdateServerLastRequest {time: now_ms()};
    tokio::spawn(async move {
        sender.send(cmd).await.expect(&*format!("Failed to send the UpdateServerLastRequest command to the client {}", client_id));
    });
}

pub async fn create_topics(path: &str, root: Arc<RwLock<Topic>>) -> Result<u64, String> {
    Topic::create_topics_gpt(path, root).await
}


pub async fn get_new_ping_reference(pings: Arc<Mutex<HashMap<u8, u128>>>, config: Arc<Config>) -> u8 {
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
 * This function return true if the client
 * is currently online.
 *
 * @param client_id u64: The checked id
 * @param clients Arc<RwLock<HashMap<u64, Sender<ClientActions>>>>: The hashmap of all connected client
 * @return bool
 */
pub async fn is_online(
    client_id: u64,
    clients: Arc<RwLock<HashMap<u64, Sender<ClientActions>>>>,
) -> bool
{
    let clients_read = clients.read().await;
    return clients_read.contains_key(&client_id);
}


const FNV_PRIME: u64 = 1099511628211;
const FNV_OFFSET: u64 = 14695981039346656037;
/**
 * This method hash the passed string
 *
 * @param s &str: the string to hash
 * @return u64: the hash of the string
 */
pub fn custom_string_hash(s: &str) -> u64 {
    let mut hash = FNV_OFFSET;
    for b in s.bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}