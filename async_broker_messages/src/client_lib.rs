// This document contain all use-full method used by the client struct.
// @author : GuicLuca (lucasguichard127@gmail.com)
// date : 22/03/2023

use std::time::{SystemTime, UNIX_EPOCH};

use tokio::sync::{oneshot};

use crate::client_lib::ClientActions::Get;
use crate::CONFIG;
use crate::config::LogLevel::Warning;
use crate::server_lib::log;
use crate::server_lib::LogSource::ClientManager;
use crate::types::{ClientSender,Responder, ServerSocket, TopicId};


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
    HandleData {
        sender: ServerSocket,
        buffer: [u8; 1024],
    },
    AddSubscribedTopic{
        topic_id: TopicId
    },
    RemoveSubscribedTopic{
        topic_id: TopicId
    },
    StartManagers{
        server_sender: ServerSocket
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
    },
    HandleTopicRequest{
        server_socket: ServerSocket,
        buffer: [u8; 1024],
        client_sender: ClientSender
    },
    HandleDisconnect{
    }
}

/**
 * Return the local time since the UNIX_EPOCH in ms
 *
 * @return u128, the current time in ms
 */
pub fn now_ms() -> u128
{
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis(); // Current time in ms
}

/**
 * This method check if the client has sent a life
 * signe (sent any request) under the heartbeat period.
 *
 * @param client_sender: ClientSender, The client channel used to fire commands
 *
 * @return bool
 */
pub async fn client_has_sent_life_sign(
    client_sender: ClientSender,
) -> bool
{
    // 1 - Get the current time
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    // 2 - Spawn a channel and get the
    // last_request_from_client by using the command
    let (tx, rx) = oneshot::channel();
    let cmd = Get {
        key: "last_request_from_client".to_string(),
        resp: tx,
    };
    match client_sender.send(cmd).await {
        Ok(_)=>{}
        Err(_)=> {
            return false;
        }
    }
    // 3 - wait for the response
    let last_client_request = rx.await.unwrap().unwrap();

    // 4 - Compute the last time the client should have sent life signe.
    // = now - Heartbeat_period (in ms)
    let should_have_give_life_sign = now - (CONFIG.heart_beat_period*1000 ) as u128;
    log(Warning, ClientManager, format!("Calcul du temps : {} > {} = {}", last_client_request, should_have_give_life_sign, last_client_request >= should_have_give_life_sign));
    // return true if the last request is sooner than the current time minus the heartbeat_period
    return last_client_request >= should_have_give_life_sign;
}