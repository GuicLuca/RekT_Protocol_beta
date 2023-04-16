// This document contain the client struct. It is where all client data
// are saved and should be computed. The structure have his personal
// manager (tokio task) that handle commands sent by the server task.
// The manager should never be blocked.
// @author : GuicLuca (lucasguichard127@gmail.com)
// date : 22/03/2023

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use tokio::sync::mpsc::{Receiver};
use tokio::time::sleep;

use crate::client_lib::{client_has_sent_life_sign, ClientActions, generate_object_id, get_object_id_type, now_ms, vec_to_u8};
use crate::client_lib::ClientActions::*;
use crate::{CLIENTS_ADDRESSES_REF, CLIENTS_SENDERS_REF, CLIENTS_STRUCTS_REF, CONFIG, ISRUNNING, OBJECT_SUBSCRIBERS_REF, OBJECTS_TOPICS_REF, PINGS_REF, TOPICS_SUBSCRIBERS_REF};
use crate::config::LogLevel::*;
use crate::datagram::*;
use crate::server_lib::*;
use crate::server_lib::LogSource::*;
use crate::types::{ClientId, ObjectId, ServerSocket};

#[derive(Debug)]
pub struct Client {
    // Identifiers
    pub id: ClientId,
    socket: SocketAddr,
    receiver: Receiver<ClientActions>,

    // Connection checker
    last_request_from_client: u128,
    last_request_from_server: u128,

    // Timestamp of the last request received from this user
    ping: u128, // client ping in ms

    // Topics
    topics: RwLock<HashSet<u64>>,
    // Contain each subscribed topic id
    requests_counter: Arc<RwLock<HashMap<u64, u32>>>, // Contain the id of the last request on a topic for scheduling
}

impl Client {
    /**
     * Only give the id and the socket of the client to create a new one
     *
     * @param id: ClientId, The server identifier of the client
     * @param socket: SocketAddr, The socket of the client (used to sent data through network)
     * @param receiver: Receiver<ClientActions>, The receiver part of the mpsc channel
     *
     * @return Client
     */
    pub fn new(
        id: ClientId,
        socket: SocketAddr,
        receiver: Receiver<ClientActions>,
    ) -> Client {
        Client {
            id,
            socket,
            receiver,
            last_request_from_client: now_ms(),
            last_request_from_server: 0,
            ping: 0,
            topics: RwLock::from(HashSet::new()),
            requests_counter: Arc::new(RwLock::from(HashMap::new())),
        }
    }

    /**
     * This method save the current time as the last client request received
     *
     * @param time: u128, the last time the client has sent a life signe in ms.
     * @param config: &Arc<Config>, The config used to log
     */
    pub async fn save_client_request_timestamp(&mut self,
                                               time: u128,
    ) {
        log(Info, ClientManager, format!("Last client request time updated for client {}", self.id));
        self.last_request_from_client = time;
    }

    /**
     * This method save the current time as the last server request sent
     *
     * @param time: u128, the last time the server has sent a request in ms.
     * @param config: &Arc<Config>, The config used to log
     */
    pub async fn save_server_request_timestamp(&mut self,
                                               time: u128,
    ) {
        log(Info, ClientManager, format!("Last server request time updated for client {}", self.id));
        self.last_request_from_server = time;
    }

    /**
     * This is the main function that receive all data of the client
     * and which return result or process logic operation on the client
     * while the connection is up.
     *
     * @param config: &Arc<Config>
     */
    pub async fn manager(&mut self)
    {
        log(Info, ClientManager, format!("Manager spawned for client {}", self.id));

        // 1 - while client is alive ...
        loop {
            // 2 - Receive actions
            while let Some(cmd) = self.receiver.recv().await {
                match cmd {
                    // This command return the value of the requested key
                    Get {
                        key,
                        resp
                    } => {
                        match key.as_str() {
                            // list of valid key under :
                            "id" => {
                                let res = self.id;
                                // Ignore errors
                                resp.send(Ok(res as u128)).expect("Error while sending response");
                            }
                            "last_request_from_client" => {
                                resp.send(Ok(self.last_request_from_client)).expect("Error while sending response");
                            }
                            _ => {
                                resp.send(Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Bad key requested : \"{}\"", key))))
                                    .expect("Error while sending response");
                            }
                        }
                    }

                    // This command get the data sent by a client and sent it to every subscribed client
                    HandleData {
                        sender,
                        buffer,
                    } => {
                        // 1 - clone local variables to give theme to the tokio task
                        let requests_counter_ref = self.requests_counter.clone();
                        let client_id = self.id;
                        // 2 - Spawn a async task
                        tokio::spawn(async move {
                            // 1 - Build the request struct for easy access members
                            let data_rq = RQ_Data::from(buffer.as_ref());
                            {
                                // 2 - Check if the request is newer than the previous one.
                                let mut write_rq_counter_ref = requests_counter_ref.write().unwrap();
                                if write_rq_counter_ref.contains_key(&data_rq.topic_id) {
                                    if data_rq.sequence_number < *write_rq_counter_ref.get(&data_rq.topic_id).unwrap()
                                        && data_rq.sequence_number > 50 // this condition allow circle id (id 0..49 will override MAX_ID)
                                    {
                                        // Request is out-dated or the topic is unknown
                                        log(Warning, DataHandler, format!("Client {} sent a data with a sequence number lower than the last one", client_id));
                                        return;
                                    }
                                }

                                // New request, update the last request id / or insert it if the topic_id is unknown
                                write_rq_counter_ref.insert(data_rq.topic_id, data_rq.sequence_number);
                                log(Info, DataHandler, format!("New data sequence for client {}.", client_id));
                            }

                            // 3 - get a list of interested clients
                            let mut interested_clients = {
                                let read_client_topics = TOPICS_SUBSCRIBERS_REF.read().await;
                                if !read_client_topics.contains_key(&data_rq.topic_id) {
                                    log(Warning, DataHandler, format!("Topic {} doesn't exist", data_rq.topic_id));
                                    return;
                                }

                                read_client_topics.get(&data_rq.topic_id).unwrap().clone()
                            };

                            // 4 - remove the source client and then sent the new data to every interested client
                            interested_clients.remove(&client_id);
                            for client in interested_clients {
                                let client_sender = {
                                    let map = CLIENTS_SENDERS_REF.read().await;
                                    map.get(&client).unwrap().clone()
                                };
                                let data = RQ_Data::new(data_rq.sequence_number, data_rq.topic_id, data_rq.data.clone());
                                let data = data.as_bytes();
                                let client_addr = {
                                    // use closure to reduce the lock lifetime
                                    *CLIENTS_ADDRESSES_REF.read().await.get(&client).unwrap()
                                };
                                let result = send_datagram(sender.clone(), &data, client_addr, client_sender).await;

                                log(Info, DataHandler, format!("Data propagation : Sent {} bytes to {}", result.unwrap(), client_addr));
                            }
                        });
                    }

                    // This command compute the ping and save the result in the local client object
                    HandlePong {
                        ping_id,
                        current_time,
                    } => {
                        let round_trip;
                        {
                            // 1 - get the mutable ref of all ping request
                            let mut pings_ref_mut = PINGS_REF.lock().await;
                            // 2 - compute the round trip
                            round_trip = (current_time - pings_ref_mut.get(&ping_id).unwrap()) / 2;
                            // 3 - free the ping_id
                            pings_ref_mut.remove(&ping_id);
                        }
                        // 4 - set the ping for the client
                        self.ping = round_trip;
                        log(Info, ClientManager, format!("There is {}ms of ping between {} and the server", round_trip, self.id));
                    }

                    // The two following command update last interaction in the client and server sens
                    UpdateServerLastRequest { time } => {
                        self.save_server_request_timestamp(time).await;
                    }
                    UpdateClientLastRequest { time } => {
                        self.save_client_request_timestamp(time).await;
                    }

                    // This command clear client data and stop client's task
                    HandleDisconnect {} => {
                        /* need to remove from :
                            1 - client_topic : remove the ic from every subscribed
                            2 - clients_ref hashmap => will stop ping_sender task and heartbeat_checker task
                            3 - clients_addresses => this address is now free for a new connection
                        */
                        // 1 - Get subscribed topics
                        let topic_ids: Vec<u64> = {
                            self.topics.read().unwrap().iter().cloned().collect() // transform the hashset to a vec
                        };

                        // last used of the channel is done : close it to cancel all pending requests
                        self.receiver.close();
                        let id = self.id;
                        tokio::spawn(async move {
                            {
                                let mut write_client_topics = TOPICS_SUBSCRIBERS_REF.write().await;
                                topic_ids.into_iter().for_each(|topic_id| {
                                    write_client_topics.get_mut(&topic_id).unwrap().remove(&id);
                                });
                            }

                            // 2&3 - Remove the id from the server memory
                            {
                                CLIENTS_SENDERS_REF.write().await.remove(&id);
                            }
                            {
                                CLIENTS_ADDRESSES_REF.write().await.remove(&id);
                            }
                            {
                                CLIENTS_STRUCTS_REF.write().await.remove(&id);
                            }
                            log(Info, Other, format!("Disconnection of client {}", id));
                        });
                    }

                    // This command start every client managers
                    StartManagers {
                        server_sender,
                    } => {

                        // heartbeat_manager :
                        let server_sender_ref = server_sender.clone();
                        let id = self.id;
                        let socket = self.socket;
                        tokio::spawn(async move {
                            heartbeat_manager(id, socket, server_sender_ref).await;
                        });
                    }

                    // This command will compute a topic request, update the client
                    // and then answer the client with a ACK or NACK.
                    HandleTopicRequest {
                        server_socket,
                        buffer,
                        client_sender,
                    } => {
                        let client_id = self.id;
                        let client_addr = self.socket;
                        tokio::spawn(async move {
                            // 1 - Init local variables
                            let topic_rq = RQ_TopicRequest::from(buffer.as_ref());
                            let result;

                            match topic_rq.action {
                                TopicsAction::SUBSCRIBE => {
                                    // 1 - check if topic exist, then insert/update the value by adding the client id to the topic
                                    let is_subscribed = {
                                        let mut topics_sub_write = TOPICS_SUBSCRIBERS_REF.write().await;
                                        let subscribers_set = topics_sub_write.entry(topic_rq.topic_id)
                                            .or_insert({
                                                // Create a new hashset and insert the client
                                                let new_hash = HashSet::new();
                                                new_hash
                                            });
                                        subscribers_set.insert(client_id)
                                    };

                                    // this is false when the client is already sub
                                    if is_subscribed {
                                        // 2 - inform the client that he's subscribed
                                        let cmd = AddSubscribedTopic { topic_ids: vec![topic_rq.topic_id] };
                                        let cmd_sender = client_sender.clone();
                                        tokio::spawn(async move {
                                            match cmd_sender.send(cmd).await {
                                                Ok(_) => {}
                                                Err(_) => {
                                                    return;
                                                }
                                            };
                                        });

                                        // 3 - respond with a topic ack
                                        result = send_datagram(
                                            server_socket.clone(),
                                            &RQ_TopicRequest_ACK::new(topic_rq.topic_id, TopicsResponse::SUCCESS_SUB).as_bytes(),
                                            client_addr,
                                            client_sender,
                                        ).await;

                                        match result {
                                            Ok(_) => {
                                                log(Info, TopicHandler, format!("{} has successfully sub the topic {}", client_id, topic_rq.topic_id));
                                            }
                                            Err(error) => {
                                                log(Info, TopicHandler, format!("Failed to send ACK to {}.\nError: {}", client_id, error));
                                            }
                                        }
                                    } else {
                                        // 2bis - Respond with an error message
                                        result = send_datagram(
                                            server_socket.clone(),
                                            &RQ_TopicRequest_NACK::new(TopicsResponse::FAILURE_SUB, "Already subscribed to {}.").as_bytes(),
                                            client_addr,
                                            client_sender,
                                        ).await;

                                        match result {
                                            Ok(_) => {
                                                log(Info, TopicHandler, format!("{} has failed sub the topic {} (ALREADY SUB)", client_id, topic_rq.topic_id));
                                            }
                                            Err(error) => {
                                                log(Info, TopicHandler, format!("Failed to send ACK to {}.\nError: {}", client_id, error));
                                            }
                                        }
                                    }
                                }
                                TopicsAction::UNSUBSCRIBE => {
                                    // get the hash of the topic
                                    // Remove it from the common array
                                    {
                                        TOPICS_SUBSCRIBERS_REF.write().await.entry(topic_rq.topic_id).and_modify(|vec| { vec.retain(|e| *e != client_id) });
                                    }

                                    // remove it from the personal client set
                                    let cmd = RemoveSubscribedTopic { topic_ids: vec![topic_rq.topic_id] };
                                    let cmd_sender = client_sender.clone();
                                    tokio::spawn(async move {
                                        match cmd_sender.send(cmd).await {
                                            Ok(_) => {}
                                            Err(_) => {
                                                return;
                                            }
                                        };
                                    });

                                    result = send_datagram(
                                        server_socket.clone(),
                                        &RQ_TopicRequest_ACK::new(topic_rq.topic_id, TopicsResponse::SUCCESS_USUB).as_bytes(),
                                        client_addr,
                                        client_sender,
                                    ).await;

                                    match result {
                                        Ok(_) => {
                                            log(Info, TopicHandler, format!("{} has successfully unsub the topic {}", client_id, topic_rq.topic_id));
                                        }
                                        Err(error) => {
                                            log(Info, TopicHandler, format!("Failed to send ACK to {}.\nError: {}", client_id, error));
                                        }
                                    }
                                }
                                TopicsAction::UNKNOWN => {}
                            }
                        });
                    }

                    // The two following task update the subscribed topics of this client
                    AddSubscribedTopic {
                        topic_ids
                    } => {
                        let mut topics_guard = self.topics.write().unwrap();
                        topic_ids.iter().for_each(move |id|{
                            topics_guard.insert(*id);
                        });
                    }
                    RemoveSubscribedTopic {
                        topic_ids
                    } => {
                        let mut topics_guard = self.topics.write().unwrap();
                        topic_ids.iter().for_each(move |id|{
                            topics_guard.remove(id);
                        });
                    }

                    // The following command handle ObjectRequest
                    HandleObjectRequest {
                        buffer,
                        server_socket,
                        client_sender,
                    } => {
                        let client_id = self.id;
                        let client_addr = self.socket;

                        tokio::spawn(async move {
                            let request = RQ_ObjectRequest::from(buffer.as_ref());
                            let new_object_id: ObjectId;
                            // Check the flag to know how to handle the request
                            match request.flags {
                                // Create the new object and sub the client to it
                                ObjectFlags::CREATE => {
                                    // Check that the id is not a broker generated objectID
                                    match get_object_id_type(request.object_id) {
                                        ObjectIdentifierType::USER_GENERATED => {
                                            let temp_id = (request.object_id & 0x3FFFFFFFFFFFFFFF) | 0x0100000000000000; // mark the old_id as a broker one
                                            // ... Then check if the broker already have it.
                                            let contains_key = {
                                                OBJECTS_TOPICS_REF.read().await.contains_key(&temp_id)
                                            };
                                            if contains_key {
                                                // Object id invalid, cancel the creation
                                                let flag: Vec<u8> = vec![0,0,0,0,0,0,0,1]; // was a create request

                                                match send_datagram(
                                                    server_socket.clone(),
                                                    &RQ_ObjectRequest_NACK::new(vec_to_u8(flag), request.object_id, "Invalid Object identifier sent. Already exist").as_bytes(),
                                                    client_addr,
                                                    client_sender,
                                                ).await {
                                                    Ok(_) => {
                                                        log(Info, ObjectHandler, format!("{} has sent an invalid object identifier (already exist) in a RQ_ObjectRequestCreate. id : {:b}", client_id, request.object_id));
                                                    }
                                                    Err(error) => {
                                                        log(Info, ObjectHandler, format!("Failed to send NACK (invalid object identifier, already exist) to {} for a RQ_ObjectRequestCreate.\nError: {}", client_id, error));
                                                    }
                                                }
                                                return; // Do not execute th following code
                                            }
                                            // Id is valid : enforce it as a new broker generated id.
                                            new_object_id = temp_id;
                                        }
                                        ObjectIdentifierType::TEMPORARY => {
                                            // Generate a new one by with the broker mark
                                            new_object_id = generate_object_id(ObjectIdentifierType::BROKER_GENERATED);
                                        }
                                        // Invalid identifier type
                                        ObjectIdentifierType::UNKNOWN | ObjectIdentifierType::BROKER_GENERATED => {
                                            let flag: Vec<u8> = vec![0,0,0,0,0,0,0,1]; // was a create request

                                            match send_datagram(
                                                server_socket.clone(),
                                                &RQ_ObjectRequest_NACK::new(vec_to_u8(flag), request.object_id, "Invalid Object identifier sent. Bad format.").as_bytes(),
                                                client_addr,
                                                client_sender,
                                            ).await {
                                                Ok(_) => {
                                                    log(Info, ObjectHandler, format!("{} has sent an invalid object identifier (bad format) in a RQ_ObjectRequestCreate. id : {:b}", client_id, request.object_id));
                                                }
                                                Err(error) => {
                                                    log(Info, ObjectHandler, format!("Failed to send NACK (invalid object identifier, bad format) to {} in a RQ_ObjectRequestCreate.\nError: {}", client_id, error));
                                                }
                                            }
                                            return; // Do not execute th following code
                                        }
                                    }// End match object_id type
                                    // The object id is valid and we can register the new object before answer the request.
                                    // Create the object :
                                    {
                                        OBJECTS_TOPICS_REF.write().await.insert(new_object_id, request.topics.clone());
                                    }

                                    // Register every topic : (create new one if they are unknown)
                                    // It add the client_id as subscriber as well
                                    {
                                        let mut topics_list_writ = TOPICS_SUBSCRIBERS_REF.write().await;
                                        request.topics.iter().for_each(move |topic_id|{
                                           topics_list_writ.entry(*topic_id)
                                               .and_modify(|subscribers|{
                                               subscribers.insert(client_id);
                                               })
                                               .or_insert(HashSet::from([client_id]));
                                        });
                                    }

                                    // Add the first object subscriber
                                    {
                                        let mut object_sub_write = OBJECT_SUBSCRIBERS_REF.write().await;
                                        let subscribers_set = object_sub_write.entry(new_object_id)
                                            .or_insert({
                                                // Create a new hashset and insert the client
                                                let new_hash = HashSet::new();
                                                new_hash
                                            });
                                        subscribers_set.insert(client_id);
                                    }

                                    // Add topics to the object creator (client)
                                    let cmd = AddSubscribedTopic {
                                        topic_ids: request.topics.into_iter().collect() // transform the hashset into Vec
                                    };
                                    let cmd_sender = client_sender.clone();
                                    tokio::spawn(async move {
                                        match cmd_sender.send(cmd).await {
                                            Ok(_) => {}
                                            Err(_) => {
                                                return;
                                            }
                                        };
                                    });

                                    // Send an ack request
                                    let flag: Vec<u8> = vec![1,0,0,0,0,0,0,1]; // was a valid create request

                                    match send_datagram(
                                        server_socket.clone(),
                                        &RQ_ObjectRequestCreate_ACK::new(vec_to_u8(flag), request.object_id, new_object_id).as_bytes(),
                                        client_addr,
                                        client_sender,
                                    ).await {
                                        Ok(_) => {
                                            log(Info, ObjectHandler, format!("{} has successfully subscribe to the object {} from a RQ_ObjectRequestCreate", client_id, new_object_id));
                                        }
                                        Err(error) => {
                                            log(Info, ObjectHandler, format!("Failed to send RQ_ObjectRequestCreate_ACK to {}.\nError: {}", client_id, error));
                                        }
                                    }
                                } // End create
                                // Update the object and update Client subscriptions
                                ObjectFlags::UPDATE => {}
                                // Delete the object and Update Client subscriptions
                                ObjectFlags::DELETE => {
                                    // Check if id type is valid
                                    if get_object_id_type(request.object_id) != ObjectIdentifierType::BROKER_GENERATED {
                                        let flag: Vec<u8> = vec![0,0,0,0,0,0,0,1]; // was a create request

                                        match send_datagram(
                                            server_socket.clone(),
                                            &RQ_ObjectRequest_NACK::new(vec_to_u8(flag), request.object_id, "Invalid Object identifier sent in a RQ_ObjectRequestDelete. Bad format.").as_bytes(),
                                            client_addr,
                                            client_sender.clone(),
                                        ).await {
                                            Ok(_) => {
                                                log(Info, ObjectHandler, format!("{} has sent an invalid object identifier (bad format) in a RQ_ObjectRequestDelete. id : {:b}", client_id, request.object_id));
                                            }
                                            Err(error) => {
                                                log(Info, ObjectHandler, format!("Failed to send NACK (invalid object identifier in a RQ_ObjectRequestDelete, bad format) to {}.\nError: {}", client_id, error));
                                            }
                                        }
                                    }
                                    // Object id is correct
                                    // Get subscribed client
                                    let clients_sub = {
                                        OBJECT_SUBSCRIBERS_REF.read().await.get(&request.object_id).unwrap().clone()
                                    };
                                    // unsubscribe every client sub to this object
                                    for subscriber_id in clients_sub {
                                        let client_sender = {
                                            CLIENTS_SENDERS_REF.read().await.get(&subscriber_id).unwrap().clone()
                                        };

                                        let cmd = RemoveSubscribedTopic {
                                            topic_ids: request.topics.clone().into_iter().collect() // transform the hashset into Vec
                                        };
                                        let cmd_sender = client_sender.clone();
                                        tokio::spawn(async move {
                                            match cmd_sender.send(cmd).await {
                                                Ok(_) => {}
                                                Err(_) => {
                                                    return;
                                                }
                                            };
                                        });

                                        let client_addr = {
                                            CLIENTS_ADDRESSES_REF.read().await.get(&subscriber_id).unwrap().clone()
                                        };

                                        let mut flag: Vec<u8> = Vec::with_capacity(8);
                                        if client_id == subscriber_id {
                                            flag.extend([1,0,0,0,0,1,0,0]); // was a valid delete request (he is the source of the deletion)
                                        }else{
                                            flag.extend([1,0,0,0,1,0,0,0]); // was a valid unsubscribe request (they are not the source of the delete)
                                        }

                                        match send_datagram(
                                            server_socket.clone(),
                                            &RQ_ObjectRequestDefault_ACK::new(vec_to_u8(flag), request.object_id).as_bytes(),
                                            client_addr,
                                            client_sender
                                        ).await {
                                            Ok(_) => {
                                                log(Info, ObjectHandler, format!("Unsubscribe notify sent to {} for the object_id : {:b}", subscriber_id, request.object_id));
                                            }
                                            Err(error) => {
                                                log(Info, ObjectHandler, format!("Failed to send Unsubscribe notify to {}.\nError: {}", subscriber_id, error));
                                            }
                                        }
                                    } // end for

                                    // Delete topics and then the object itself
                                    {
                                        let keys_to_remove = OBJECTS_TOPICS_REF.read().await.get(&request.object_id).unwrap().clone();
                                        let mut topics_sub_ref =TOPICS_SUBSCRIBERS_REF.write().await;
                                        keys_to_remove.iter().for_each(move |id|{
                                            topics_sub_ref.remove(id);
                                        });
                                    }
                                    {
                                        OBJECTS_TOPICS_REF.write().await.remove(&request.object_id);
                                    }

                                }
                                // Subscribe the client to all topics of this object
                                ObjectFlags::SUBSCRIBE => {}
                                // Unsubscribe the client to all topics of this object
                                ObjectFlags::UNSUBSCRIBE => {}
                                // Respond with an Error
                                ObjectFlags::UNKNOWN => {
                                    let flag: Vec<u8> = vec![0,0,0,0,0,0,0,0]; // was an invalid unknown request

                                    match send_datagram(
                                        server_socket.clone(),
                                        &RQ_ObjectRequest_NACK::new(vec_to_u8(flag), request.object_id, "Invalid flag sent").as_bytes(),
                                        client_addr,
                                        client_sender,
                                    ).await {
                                        Ok(_) => {
                                            log(Info, ObjectHandler, format!("{} has sent invalid flags for an object request. flags: {}", client_id, u8::from(request.flags)));
                                        }
                                        Err(error) => {
                                            log(Info, ObjectHandler, format!("Failed to send NACK (invalid flags) to {}.\nError: {}", client_id, error));
                                        }
                                    }
                                }
                            }
                        });
                    }
                } // end match
            }
        }
    }
}

/**
 * This function check last request time of the client and the server and sent heartbeat
 * if the time is over a limit (see config.toml).
 * If no life signal get during 4 heartbeat period : the server disconnect the client.
 *
 * @param id: ClientId
 * @param socket: SocketAddr, The client address.
 * @param server_sender: ServerSocket, the server socket used to exchange data
 */
pub async fn heartbeat_manager(
    id: ClientId,
    socket: SocketAddr,
    server_sender: ServerSocket,
)
{
    log(Info, HeartbeatChecker, format!("HeartbeatChecker spawned for {}", id));
    // 1 - Init local variables
    let mut missed_heartbeats: u8 = 0; // used to count how many HB are missing
    let client_sender = {
        CLIENTS_SENDERS_REF.read().await.get(&id).unwrap().clone()
    };

    // Init the server status
    let mut server_is_running = {
        *ISRUNNING.read().await
    };

    // 2 - Loop while server is running and client is online
    while server_is_running && is_online(id).await {
        // 3 - waite for the heartbeat period
        sleep(Duration::from_secs(CONFIG.heart_beat_period as u64)).await;


        // 4 - check if client has sent heartbeat
        if !client_has_sent_life_sign(client_sender.clone()).await {
            //  5 - increase the missing packet
            missed_heartbeats += 1;
            log(Info, HeartbeatChecker, format!("{} hasn't sent life signs {} times", id, missed_heartbeats));
            if missed_heartbeats == 2 {
                // 6 - send an heartbeat request
                let result = send_datagram(
                    server_sender.clone(),
                    &RQ_Heartbeat_Request::new().as_bytes(),
                    socket,
                    client_sender.clone(),
                ).await;


                match result {
                    Ok(bytes) => {
                        log(Info, HeartbeatChecker, format!("Send {} bytes (RQ_Heartbeat_Request) to {}", bytes, id));
                    }
                    Err(error) => {
                        log(Error, HeartbeatChecker, format!("Failed to send RQ_HeartBeat_Request to {}. \nError: {}", id, error));
                    }
                }
            } else if missed_heartbeats == 4 {
                // 7 - No Heartbeat for 4 period = end the client connection.
                // Send shutdown request
                let result = send_datagram(
                    server_sender.clone(),
                    &RQ_Shutdown::new(EndConnexionReason::SHUTDOWN).as_bytes(),
                    socket,
                    client_sender.clone(),
                ).await;

                match result {
                    Ok(bytes) => {
                        log(Info, HeartbeatChecker, format!("Send {} bytes (RQ_Heartbeat_Request) to {}", bytes, id));
                    }
                    Err(error) => {
                        log(Error, HeartbeatChecker, format!("Failed to send RQ_Shutdown to {}. \nError: {}", id, error));
                    }
                }

                // 8 - Remove the client from the main array
                // client's task will end automatically
                let cmd = HandleDisconnect {};
                let sender_ref = client_sender.clone();
                match sender_ref.send(cmd).await {
                    Ok(_) => {}
                    Err(_) => {
                        return;
                    }
                };
            }
        } else {
            // 5bis - reset flags variable
            missed_heartbeats = 0;
            let result = send_datagram(
                server_sender.clone(),
                &RQ_Heartbeat::new().as_bytes(),
                socket,
                client_sender.clone(),
            ).await;

            match result {
                Ok(_) => {
                    log(Info, HeartbeatChecker, format!("Respond to client bytes (HeartBeat) to {}", id));
                }
                Err(error) => {
                    log(Error, HeartbeatChecker, format!("Failed to send RQ_HeartBeat to {}.\nError: {}", id, error));
                }
            }
        }

        // Update the server status before looping again
        server_is_running = {
            *ISRUNNING.read().await
        };
    } // End while

    // 9 - End the task
    log(Info, HeartbeatChecker, format!("Heartbeat checker destroyed for {}", id));
}