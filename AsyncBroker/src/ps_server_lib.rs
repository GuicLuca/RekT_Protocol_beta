use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use rand::Rng;

pub fn already_connected(ip: &IpAddr, clients : HashMap<u64, SocketAddr>) -> (bool, u64) {
    for (key, value) in clients {
        if ip == &value.ip() {
            return (true, key);
        }
    }
    return (false, 0);
}

pub fn get_new_id(clients: HashMap<u64, SocketAddr>) -> u64 {
    let mut rng = rand::thread_rng();
    let mut nb: u64 = rng.gen();
    if clients.contains_key(&nb) {
        nb = get_new_id(clients);
    }
    return nb;
}