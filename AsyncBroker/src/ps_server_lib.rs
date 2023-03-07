use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::Rng;
use tokio::net::UdpSocket;
use tokio::sync::{Mutex, MutexGuard};
use crate::ps_datagram_structs::*;
use crate::topic_v2::TopicV2;


pub async fn already_connected<'a>(
    ip: &'a IpAddr,
    clients: MutexGuard<'a, HashMap<u64, SocketAddr>>
) -> (bool, u64) {
    for (key, value) in clients.iter() {
        if ip == &value.ip() {
            return (true, *key);
        }
    }
    return (false, 0);
}

pub fn get_new_id(clients: MutexGuard<HashMap<u64, SocketAddr>>) -> u64 {
    let mut rng = rand::thread_rng();
    let mut nb: u64 = rng.gen();
    if clients.contains_key(&nb) {
        nb = get_new_id(clients);
    }
    return nb;
}

fn get_new_ping_id(pings: MutexGuard<HashMap<u8, u128>>) -> u8 {
    let mut rng = rand::thread_rng();
    let mut nb: u8 = rng.gen();
    if pings.contains_key(&nb) {
        nb = get_new_ping_id(pings);
    }
    return nb;
}

pub async fn get_client_id(
    src : &SocketAddr,
    clients : Arc<Mutex<HashMap<u64, SocketAddr>>>
) -> Option<u64> {
    for (key, val) in clients.lock().await.iter() {
        if val == src {
            return Some(*key);
        }
    }
    None
}

pub async fn handle_connect(src: SocketAddr, clients : Arc<Mutex<HashMap<u64, SocketAddr>>>, socket : Arc<UdpSocket>) {
    let (is_connected, current_id) = already_connected(&src.ip(),clients.lock().await).await;
    let uuid;
    let result;
    if is_connected {
        uuid = current_id;
        println!("[Server Handler] {} was already a client, UUID : {}", src.ip(), uuid);
    } else {
        uuid = get_new_id(clients.lock().await);
        println!("[Server Handler] {} is now a client, UUID : {}", src.ip(), uuid);
        let mut map = clients.lock().await;
        map.insert(uuid, src);
    }
    let datagram = &RQ_Connect_ACK_OK::new(uuid, 1).as_bytes();
    println!("[Server Handler] Connect ack OK sent. Datagram : {:?}",datagram);
    result = socket.send_to(datagram, src).await;
    match result {
        Ok(bytes) => {
            println!("[Server Handler] Send {} bytes to {}", bytes, src.ip());
        }
        Err(_) => {
            println!("[Server Handler] Failed to send Connect ACK to {}", src.ip());
        }
    }
}

pub async fn create_topics(path: &str, root : Arc<Mutex<TopicV2>>) -> u64 {
    TopicV2::create_topicsGPT(path, root.lock().await)
}


pub async fn get_new_ping_reference(pings : Arc<Mutex<HashMap<u8, u128>>>) -> u8{
    let key = get_new_ping_id(pings.lock().await);

    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis(); // Current time in ms

    pings.lock().await.insert(key, time);
    println!("[Server Ping] New ping reference created. Id : {}", key);
    return key;
}

pub async fn handle_pong(
    client_id : u64,
    ping_id : u8,
    current_time : u128,
    pings_ref: Arc<Mutex<HashMap<u8, u128>>>,
    clients_ping: Arc<Mutex<HashMap<u64, u128>>>
){
    // 1 - get the mutable ref of all ping request
    let mut pings_ref_mut = pings_ref.lock().await;
    // 2 - compute the round trip
    let round_trip = (current_time - pings_ref_mut.get(&ping_id).unwrap()) /2;
    // 3 - free the ping_id
    pings_ref_mut.remove(&ping_id);
    // 4 - set the ping for the client_id
    clients_ping.lock().await.entry(client_id)
        .and_modify(|v| *v = round_trip)
        .or_insert(round_trip);
    println!("[Server Ping] There is {}ms of ping between {} and the server", round_trip, client_id);
}