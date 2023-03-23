// This document contain the client struct. It is where all client data
// are saved and should be computed.
// @author : GuicLuca (lucasguichard127@gmail.com)
// date : 22/03/2023

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{RwLock};
use crate::datagram::MessageType;
use crate::topic::Topic;

pub struct Client {
    // Identifiers
    id: u64,
    socket: SocketAddr,

    // Requests
    requests_state: RwLock<HashMap<MessageType,u8>>, // Contain the id of the last request of a type for scheduling
    request_buffer: RwLock<HashMap<MessageType,[[u8;1024]; 5]>>, // Array containing the 5 last request of a type to do interpolation

    // Connection checker
    last_request: u128, // Timestamp of the last request received from this user
    missed_heartbeats: u8, // amount of heartbeats_request no received
    ping: u128, // client ping in ms

    // Topics
    topics: Vec<Topic>
}