use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
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
            println!("[Server Handler] Failed to send Connect ACK to {}", src);
        }
    }
}

pub async fn create_topics(path: &str, root : Arc<Mutex<TopicV2>>) -> u64 {
    TopicV2::create_topicsGPT(path, root.lock().await)
}