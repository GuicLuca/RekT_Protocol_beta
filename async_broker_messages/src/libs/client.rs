// This document contain all use-full method used by the client struct.
// @author : GuicLuca (lucasguichard127@gmail.com)
// date : 22/03/2023

use std::collections::HashSet;
use std::net::{IpAddr, SocketAddr};
use std::time::{SystemTime, UNIX_EPOCH};
use rand::Rng;

use tokio::sync::{oneshot};
use crate::{CLIENTS_ADDRESSES_REF, CLIENTS_SENDERS_REF, CLIENTS_STRUCTS_REF, CONFIG, OBJECT_SUBSCRIBERS_REF, OBJECTS_TOPICS_REF, TOPICS_SUBSCRIBERS_REF};

use crate::enums::client_actions::ClientActions::{AddSubscribedObject, AddSubscribedTopic, Get, RemoveSubscribedObject, RemoveSubscribedTopic};
use crate::enums::log_level::LogLevel::{Error, Info};
use crate::enums::log_source::LogSource::{HeartbeatChecker, ObjectHandler, Other};
use crate::libs::common::{log, now_ms};
use crate::libs::types::{ClientId, ClientSender, ObjectId};




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
    let now = now_ms();

    // 2 - Spawn a channel and get the
    // last_request_from_client by using the command
    let (tx, rx) = oneshot::channel();
    let cmd = Get {
        key: "last_request_from_client".to_string(),
        resp: tx,
    };
    match client_sender.send(cmd).await {
        Ok(_)=>{}
        Err(err)=> {
            log(Error, HeartbeatChecker, format!("Failed to send a Get command to a a struct in method \"client_has_sent_life_sign\". Error:\n{}", err));
            return false;
        }
    }
    // 3 - wait for the response
    let last_client_request = match rx.await {
        Ok(result) => {
            match result {
                Ok(result) => {
                    result
                }
                Err(err) => {
                    log(Error, HeartbeatChecker, format!("Failed to Get last_request_from_client in method \"client_has_sent_life_sign\". Error:\n{}", err));
                    0 // return a default value to let the flow continuing
                }
            }
        }
        Err(err) => {
            log(Error, HeartbeatChecker, format!("Failed to receive the result of a Get command in method \"client_has_sent_life_sign\". Error:\n{}", err));
            0 // return a default value to let the flow continuing
        }
    };

    // 4 - Compute the last time the client should have sent life signe.
    // = now - Heartbeat_period (in ms)
    let should_have_give_life_sign = now - (CONFIG.heart_beat_period*1000 ) as u128;
    log(Info, HeartbeatChecker, format!("Computing round-trip time : {} > {} = {}", last_client_request, should_have_give_life_sign, last_client_request >= should_have_give_life_sign));
    // return true if the last request is sooner than the current time minus the heartbeat_period
    return last_client_request >= should_have_give_life_sign;
}

/**
 * This method is a shortcut to subscribe a
 * client to an object. It will update global and
 * local hashsets in one function.
 *
 * @param object_id: ObjectId, The object identifier
 * @param client_id: ClientId, The client identifier
 * @param client_sender: ClientSender, The client sender used to send command through
 *
 * @return Result<(), String>
 */
pub async fn subscribe_client_to_object<'a>(
    object_id: ObjectId,
    client_id: ClientId,
    client_sender: ClientSender,
) -> Result<(), String>
{

    let topics = {
        match OBJECTS_TOPICS_REF.read().await.get(&object_id) {
            None => {
                return Err(format!("Can't find the object id {} in OBJECTS_TOPICS_REF.", object_id));
            }
            Some(topics) => {
                topics.clone()
            }
        }
    };

    // 1 - add topics to the client
    let cmd = AddSubscribedTopic {
        topic_ids: topics.into_iter().collect() // transform the hashset into Vec
    };
    let cmd_sender = client_sender.clone();
    tokio::spawn(async move {
        match cmd_sender.send(cmd).await {
            Ok(_) => {}
            Err(err) => {
                log(Error, ObjectHandler, format!("Failed to send AddSubscribedTopic command in method \"subscribe_client_to_object\". Error:\n{}", err));
                return;
            }
        };
    });
    // 2 - add object to the client
    let cmd = AddSubscribedObject {
        object_id
    };

    tokio::spawn(async move {
        match client_sender.send(cmd).await {
            Ok(_) => {}
            Err(err) => {
                log(Error, ObjectHandler, format!("Failed to send AddSubscribedObject command in method \"subscribe_client_to_object\". Error:\n{}", err));
                return;
            }
        };
    });

    // 2 - Add the client as object subscriber
    {
        let mut object_sub_write = OBJECT_SUBSCRIBERS_REF.write().await;
        let subscribers_set = object_sub_write.entry(object_id)
            .or_insert({
                // Create a new hashset and insert the client
                let new_hash = HashSet::new();
                new_hash
            });
        subscribers_set.insert(client_id);
    }
    // Method has successfully ran
    Ok(())
}

/**
 * This method is a shortcut to unsubscribe a
 * client to an object. It will update global and
 * local hashsets in one function.
 *
 * @param object_id: ObjectId, The object identifier
 * @param client_id: ClientId, The client identifier
 * @param client_sender: ClientSender, The client sender used to send command through
 *
 * @return Result<(), String>
 */
pub async fn unsubscribe_client_to_object(
    client_id: ClientId,
    object_id: ObjectId,
    client_sender: ClientSender
)-> Result<(), String>
{
    // 1 - Get client and object information
    let topics = {
        match OBJECTS_TOPICS_REF.read().await.get(&object_id) {
            None => {
                return Err(format!("Can't find the object id {} in OBJECTS_TOPICS_REF.", object_id));
            }
            Some(topics) => {
                topics.clone()
            }
        }
    };

    // 2 - remove topics from the client struct
    let cmd = RemoveSubscribedTopic {
        topic_ids: topics.into_iter().collect() // transform the hashset into Vec
    };
    let cmd_sender = client_sender.clone();
    tokio::spawn(async move {
        match cmd_sender.send(cmd).await {
            Ok(_) => {}
            Err(err) => {
                log(Error, ObjectHandler, format!("Failed to send RemoveSubscribedTopic command in method \"unsubscribe_client_to_object\". Error:\n{}", err));
                return;
            }
        };
    });


    // 3 - remove object to the client
    let cmd = RemoveSubscribedObject {
        object_id
    };
    tokio::spawn(async move {
        match client_sender.send(cmd).await {
            Ok(_) => {}
            Err(err) => {
                log(Error, ObjectHandler, format!("Failed to send RemoveSubscribedObject command in method \"unsubscribe_client_to_object\". Error:\n{}", err));
                return;
            }
        };
    });

    // 4 - Remove the client of the object
    {
        OBJECT_SUBSCRIBERS_REF.write().await.entry(object_id)
            .and_modify(|subscribers|{
                subscribers.remove(&client_id);
            });
    }

    Ok(()) // method has successfully ran
}

/**
 *  This method is an helper to get a client sender
 * from a client id.
 *
 * @return Result<ClientSender, ()>
 */
pub async fn get_client_sender(client_id: ClientId) -> Result<ClientSender, ()>
{
    let map = CLIENTS_SENDERS_REF.read().await;
    match map.get(&client_id) {
        None => {
            log(Error, Other, format!("Can't find client sender from the client id {}", client_id));
            try_remove_client_from_set(client_id).await;
            Err(())
        }
        Some(value) => {
            Ok(value.clone())
        }
    }
}

/**
 *  This method is an helper to get a client address
 * from a client id.
 *
 * @return Result<SocketAddr, ()>
 */
pub async fn get_client_addr(client_id: ClientId) -> Result<SocketAddr, ()>
{
    let map = CLIENTS_ADDRESSES_REF.read().await;
    match map.get(&client_id) {
        None => {
            log(Error, Other, format!("Can't find client address from the client id {}", client_id));
            try_remove_client_from_set(client_id).await;
            Err(())
        }
        Some(value) => {
            Ok(value.clone())
        }
    }
}

/**
 * This method return true if the client was already connected,
 *  it return the old client id too.
 *
 * @param ip: &IpAddr, ip address of the tested client
 *
 * @return (bool, ClientId)
 */
pub async fn already_connected(
    ip: & IpAddr,
) -> (bool, ClientId) {
    // Use iterator for performance
    CLIENTS_ADDRESSES_REF.read().await
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
 * This function return true if the client is currently online.
 *
 * @param client_id: ClientId,  The checked client identifier
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
/**
 * This method will try to remove the client id from all common sets
 *
 * @param client_id: ClientId, The client to remove from common hashsets
 */
pub async fn try_remove_client_from_set(client_id: ClientId){
    // 1 - Client sender
    {
        CLIENTS_SENDERS_REF.write().await.remove(&client_id);
    }
    // 2 - Client address
    {
        CLIENTS_ADDRESSES_REF.write().await.remove(&client_id);
    }
    // 3 - Client struct
    {
        CLIENTS_STRUCTS_REF.write().await.remove(&client_id);
    }
    // 4 - Client sender
    {
        CLIENTS_SENDERS_REF.write().await.remove(&client_id);
    }
    // 5 - Topics subscriber
    {
        let mut topics_list_writ = TOPICS_SUBSCRIBERS_REF.write().await;

        topics_list_writ.iter_mut().for_each(|(_, topic_subscribers)| {
            topic_subscribers.remove(&client_id);
        });
    }
    // 6 - Objects subscriber
    {
        let mut objects_list_writ = OBJECT_SUBSCRIBERS_REF.write().await;

        objects_list_writ.iter_mut().for_each(|(_, object_subscribers)| {
            object_subscribers.remove(&client_id);
        });
    }
}