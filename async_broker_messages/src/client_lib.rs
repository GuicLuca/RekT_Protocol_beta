// This document contain all use-full method used by the client struct.
// @author : GuicLuca (lucasguichard127@gmail.com)
// date : 22/03/2023

use std::collections::HashSet;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::sync::{oneshot};

use crate::client_lib::ClientActions::{AddSubscribedObject, AddSubscribedTopic, Get, RemoveSubscribedObject, RemoveSubscribedTopic};
use crate::{CLIENTS_ADDRESSES_REF, CLIENTS_SENDERS_REF, CONFIG, OBJECT_SUBSCRIBERS_REF, OBJECTS_TOPICS_REF};
use crate::config::LogLevel::{Error, Info};
use crate::datagram::ObjectIdentifierType;
use crate::server_lib::{log, try_remove_client_from_set};
use crate::server_lib::LogSource::{ClientManager, HeartbeatChecker, ObjectHandler, Other};
use crate::types::{ClientId, ClientSender, ObjectId, PingId, Responder, ServerSocket, TopicId};


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
        topic_ids: Vec<TopicId>
    },
    RemoveSubscribedTopic{
        topic_ids: Vec<TopicId>
    },
    AddSubscribedObject{
        object_id: ObjectId
    },
    RemoveSubscribedObject{
        object_id: ObjectId
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
        ping_id: PingId, // The ping request that is answered
        current_time: u128, // Server time when the request has been received
    },
    HandleTopicRequest{
        server_socket: ServerSocket,
        buffer: [u8; 1024],
        client_sender: ClientSender
    },
    HandleDisconnect{
    },
    HandleObjectRequest{
        buffer: [u8; 1024],
        server_socket: ServerSocket,
        client_sender: ClientSender,
    }
}

/**
 * Return the local time since the UNIX_EPOCH in ms
 *
 * @return u128, the current time in ms
 */
pub fn now_ms() -> u128
{
    match SystemTime::now().duration_since(UNIX_EPOCH){
        Ok(dur) => {
            // return the duration as millisecond
            dur.as_millis()
        }
        Err(err) => {
            log(Error, Other, format!("Failed to get duration since UNIX_EPOCH in now_ms. Error:\n{}", err));
            0 // return a default value to let the flow continuing
        }
    }
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
    log(Info, ClientManager, format!("Computing round-trip time : {} > {} = {}", last_client_request, should_have_give_life_sign, last_client_request >= should_have_give_life_sign));
    // return true if the last request is sooner than the current time minus the heartbeat_period
    return last_client_request >= should_have_give_life_sign;
}

/**
 * This method return the u8 image of the
 * given bitfields. The bitfield must be in LE endian
 *
 * @param bitfields: Vec<u8>
 *
 * @return u8
 */
pub fn vec_to_u8(bitfield: Vec<u8>) -> u8{
    (bitfield[0] << 7) | (bitfield[1] << 6) | (bitfield[2] << 5) | (bitfield[3] << 4) | (bitfield[4] << 3) | (bitfield[5] << 2) | (bitfield[6] << 1) | (bitfield[7] << 0)
}

/**
 * This method return the type of the
 * object id according to the two MSB
 *
 * @param id: ObjectId
 *
 * @return ObjectIdentifierType
 */
pub fn get_object_id_type(id: ObjectId) -> ObjectIdentifierType {
    // Use bitwise operator to get two MSB ->|XX______|
    let msb = (id >> 62) & 0b11;
    match msb {
        0b00 => {
            ObjectIdentifierType::USER_GENERATED
        }
        0b01 => {
            ObjectIdentifierType::BROKER_GENERATED
        }
        0b10 => {
            ObjectIdentifierType::TEMPORARY
        }
        _ => {
            ObjectIdentifierType::UNKNOWN
        }
    }
}

/**
 * This method generate a new object id and set
 * the two MSB according to the type needed.
 * /!\ Unknown type is not allow and will return 0.
 *
 * @param id_type: ObjectIdentifierType
 *
 * @return ObjectId or 0
 */
pub fn generate_object_id(id_type: ObjectIdentifierType) -> ObjectId {
    // 1 - Get the current time
    let now = match SystemTime::now().duration_since(UNIX_EPOCH){
        Ok(dur) => {
            // return the duration as millisecond
            dur.as_nanos() as u64
        }
        Err(err) => {
            log(Error, ObjectHandler, format!("Failed to get duration since UNIX_EPOCH in \"generate_object_id\". Error:\n{}", err));
            0 // return a default value to let the flow continuing
        }
    };



    let u64_with_msb_00 = now & 0x3FFFFFFFFFFFFFFF; // the mask allow to set two MSB to 00 to rewrite them after
    // 2 - set é MSB according to the type
    match id_type {
        ObjectIdentifierType::USER_GENERATED => {
            u64_with_msb_00 | 0x0000000000000000
        }
        ObjectIdentifierType::BROKER_GENERATED => {
            u64_with_msb_00 | 0x0100000000000000
        }
        ObjectIdentifierType::TEMPORARY => {
            u64_with_msb_00 | 0x1000000000000000
        }
        _ => 0
    }
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
 * This method ensure the given objectId
 * is a broker generated id and it still
 * exist.
 *
 * @param object_id: ObjectId, The object identifier
 *
 * @return bool
 */
pub async fn is_object_id_valid(
    object_id: ObjectId
) -> bool
{
    let existing_id = {
        OBJECT_SUBSCRIBERS_REF.read().await.get(&object_id).is_some()
    };

    return get_object_id_type(object_id) != ObjectIdentifierType::BROKER_GENERATED && existing_id;
}

/**
 * This method return a tuple off two vec containing
 * added and removed values.
 *
 * @param new_set: &HashSet<TopicId>, The new set containing incoming values
 * @param current_set: &HashSet<TopicId>, the current set containing actual values
 *
 * @return added_values, removed_values: (Vec<TopicId>, Vec<TopicId>): two vectors containing differences from the original set
 */
pub fn diff_hashsets(new_set: &HashSet<TopicId>, current_set: &HashSet<TopicId>) -> (Vec<TopicId>, Vec<TopicId>) {
    let added_values = new_set.difference(current_set).cloned().collect();
    let removed_values = current_set.difference(new_set).cloned().collect();
    (added_values, removed_values)
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
