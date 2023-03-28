// This document contain all use-full method used by the client struct.
// @author : GuicLuca (lucasguichard127@gmail.com)
// date : 22/03/2023

use std::collections::{HashMap, HashSet};
use std::io::Error;
use std::net::SocketAddr;
use std::sync::{Arc};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;

use tokio::sync::{Mutex, oneshot, RwLock};
use tokio::sync::mpsc::Sender;
use crate::client::Client;
use crate::client_lib::ClientActions::Get;
use crate::config::Config;
use crate::config::LogLevel::Warning;
use crate::server_lib::log;
use crate::server_lib::LogSource::ClientManager;
use crate::topic::Topic;

// ===================
//   Common used type
// ===================
type Responder<T> = oneshot::Sender<Result<T, Error>>;


/**
 * The ClientActions enum contain each action a user can do.
 * It will be used by the server task to ask the client struct
 * to execute something.
 */
#[derive(Debug)]
pub enum ClientActions {
    Get {
        key: String,
        resp: Responder<u128>,
    },
    GetTopics{
        resp: Responder<HashSet<u64>>
    },
    AddSubscribedTopic{
        topic_id: u64
    },
    RemoveSubscribedTopic{
        topic_id: u64
    },
    StartManagers{
        clients: Arc<tokio::sync::RwLock<HashMap<u64, Sender<ClientActions>>>>,
        topics_subscribers: Arc<tokio::sync::RwLock<HashMap<u64, HashSet<u64>>>>,
        clients_addresses: Arc<tokio::sync::RwLock<HashMap<u64, SocketAddr>>>,
        clients_structs: Arc<RwLock<HashMap<u64, Arc<tokio::sync::Mutex<Client>>>>>,
        b_running: Arc<bool>,
        server_sender: Arc<UdpSocket>
    },
    UpdateServerLastRequest{
        time: u128
    },
    UpdateClientLastRequest{
        time: u128
    },
    HandlePong {
        ping_id: u8, // The ping request that is answered
        current_time: u128, // Server time when the request has been received
        pings_ref: Arc<Mutex<HashMap<u8, u128>>>, // contain all ping request sent by the server
    },
    HandleTopicRequest{
        server_socket: Arc<UdpSocket>,
        buffer: [u8; 1024],
        topics_subscribers: Arc<RwLock<HashMap<u64, HashSet<u64>>>>,
        root_ref: Arc<RwLock<Topic>>,
        client_sender: Sender<ClientActions>
    },
    HandleDisconnect{
        topics_subscribers: Arc<RwLock<HashMap<u64, HashSet<u64>>>>,
        clients_ref: Arc<RwLock<HashMap<u64, Sender<ClientActions>>>>,
        clients_addresses: Arc<RwLock<HashMap<u64, SocketAddr>>>,
        clients_structs: Arc<RwLock<HashMap<u64, Arc<Mutex<Client>>>>>,
        client_sender: Sender<ClientActions>
    }
}

pub fn now_ms() -> u128
{
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis(); // Current time in ms
}

pub async fn client_has_sent_life_sign(
    client_sender: Sender<ClientActions>,
    config: Arc<Config>
) -> bool
{
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let (tx, rx) = oneshot::channel();

    let cmd = Get {
        key: "last_request_from_client".to_string(),
        resp: tx,
    };


    client_sender.send(cmd).await.unwrap();
    let last_client_request = rx.await.unwrap().unwrap();

    let should_have_give_life_sign = now - (config.heart_beat_period*1000 ) as u128;
    log(Warning, ClientManager, format!("Calcule du temps : {} > {} = {}", last_client_request, should_have_give_life_sign, last_client_request >= should_have_give_life_sign), config);
    // return true if the last request is sooner than the current time minus the heartbeat_period
    return last_client_request >= should_have_give_life_sign;
}