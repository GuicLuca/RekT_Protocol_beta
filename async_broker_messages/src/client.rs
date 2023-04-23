// This document contain the client struct. It is where all client data
// are saved and should be computed. The structure have his personal
// manager (tokio task) that handle commands sent by the server task.
// The manager should never be blocked.
// @author : GuicLuca (lucasguichard127@gmail.com)
// date : 22/03/2023

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc::{Receiver};
use tokio::sync::RwLock;
use tokio::time::sleep;

use crate::client_lib::{client_has_sent_life_sign, ClientActions, diff_hashsets, generate_object_id, get_client_addr, get_client_sender, get_object_id_type, is_object_id_valid, now_ms, subscribe_client_to_object, unsubscribe_client_to_object, vec_to_u8};
use crate::client_lib::ClientActions::*;
use crate::{CLIENTS_ADDRESSES_REF, CLIENTS_SENDERS_REF, CLIENTS_STRUCTS_REF, CONFIG, ISRUNNING, OBJECT_SUBSCRIBERS_REF, OBJECTS_TOPICS_REF, PINGS_REF, TOPICS_SUBSCRIBERS_REF};
use crate::config::LogLevel::*;
use crate::datagram::*;
use crate::datagram::EndConnexionReason::SYNC_ERROR;
use crate::server_lib::*;
use crate::server_lib::LogSource::*;
use crate::types::{ClientId, ObjectId, ServerSocket, TopicId};

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
    topics: RwLock<HashSet<TopicId>>,
    // Objects
    objects: RwLock<HashSet<ObjectId>>,
    // Contain each subscribed topic id
    requests_counter: Arc<RwLock<HashMap<TopicId, u32>>>, // Contain the id of the last request on a topic for scheduling
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
            objects: RwLock::from(HashSet::new()),
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
                            let data_rq = match RQ_Data::try_from(buffer.as_ref()){
                                Ok(request) => {
                                    request
                                }
                                Err(err) => {
                                    log(Error, DataHandler, format!("Failed to construct the data request structure. Request:\n{:?} \n\nError:\n{}", buffer, err));
                                    return;
                                }
                            };

                            {
                                // 2 - Check if the request is newer than the previous one.
                                let mut write_rq_counter_ref = requests_counter_ref.write().await;
                                match write_rq_counter_ref.get(&data_rq.topic_id){
                                    None => {
                                        // Unknown sequence number so it must be valid
                                    }
                                    Some(current_sequence) => {
                                        if data_rq.sequence_number < *current_sequence
                                            && data_rq.sequence_number > 50 // this condition allow circle id (id 0..49 will override MAX_ID)
                                        {
                                            // Request is out-dated or the topic is unknown
                                            log(Warning, DataHandler, format!("Client {} sent a data with a sequence number lower than the last one", client_id));
                                            return;
                                        }
                                    }
                                }

                                // New request, update the last request id / or insert it if the topic_id is unknown
                                write_rq_counter_ref.insert(data_rq.topic_id, data_rq.sequence_number);
                                log(Info, DataHandler, format!("New data sequence for client {}.", client_id));
                            }

                            // 3 - get a list of interested clients
                            let mut interested_clients = {
                                match TOPICS_SUBSCRIBERS_REF.read().await.get(&data_rq.topic_id) {
                                    None => {
                                        log(Warning, DataHandler, format!("Topic {} doesn't exist", data_rq.topic_id));
                                        return;
                                    }
                                    Some(subscribers) => {
                                        subscribers.clone()
                                    }
                                }
                            };

                            // 4 - remove the source client and then sent the new data to every interested client
                            interested_clients.remove(&client_id);
                            for client in interested_clients {
                                let client_addr = match get_client_addr(client).await {
                                    Ok(value) => {value}
                                    Err(_) => {
                                        // Can't find client address : disconnect the client
                                        log(Error, DataHandler, format!("Can't find client address. Disconnect the client but can't send Shutdown error."));
                                        return;
                                    }
                                };
                                let client_sender = match get_client_sender(client).await {
                                    Ok(sender) => {sender}
                                    Err(_) => {
                                        // sender can't be found : send a shutdown for sync error
                                        match sender.send_to(
                                            &RQ_Shutdown::new(SYNC_ERROR).as_bytes(),
                                            client_addr
                                        ).await {
                                            Err(err) => {
                                                log(Error, DataHandler, format!("Failed to send connexion error to {}. The client has common hashset sync issue. Error:\n{}", client, err));
                                            }
                                            _ => {}
                                        }
                                        return;
                                    }
                                };
                                let data = RQ_Data::new(data_rq.sequence_number, data_rq.topic_id, data_rq.data.clone());
                                let data = data.as_bytes();

                                match send_datagram(sender.clone(), &data, client_addr, client_sender).await {
                                    Ok(bytes) => {
                                        log(Info, DataHandler, format!("Data propagation : Sent {} bytes to {}", bytes, client));
                                    }
                                    Err(err) => {
                                        log(Error, DataHandler, format!("Data propagation : Failed to send data to {}. Error:\n{}", client_addr, err));
                                    }
                                }
                            } // end for
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
                            round_trip = match pings_ref_mut.get(&ping_id){
                                None => {
                                    log(Warning, ClientManager, format!("Client {} has tried to answer a wrong ping id. Id sent : {}", self.id, ping_id));
                                    continue;
                                }
                                Some(time_reference) => {
                                    (current_time - time_reference) / 2
                                }
                            };
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
                        // 1 - Get subscribed topics & objects
                        let topic_ids: Vec<u64> = {
                            self.topics.read().await.iter().cloned().collect() // transform the hashset to a vec
                        };
                        let object_ids: Vec<u64> = {
                            self.objects.read().await.iter().cloned().collect() // transform the hashset to a vec
                        };

                        // last used of the channel is done : close it to cancel all pending requests
                        self.receiver.close();
                        let id = self.id;
                        tokio::spawn(async move {
                            // 2 - remove the client form his subscriptions (topic and objects)
                            {
                                let mut write_client_topics = TOPICS_SUBSCRIBERS_REF.write().await;
                                topic_ids.into_iter().for_each(|topic_id| {
                                    match write_client_topics.get_mut(&topic_id) {
                                        None => {
                                            // Skip
                                        }
                                        Some(subscribers) => {
                                            subscribers.remove(&id);
                                        }
                                    }
                                });
                            }
                            {
                                let mut write_client_objects = OBJECT_SUBSCRIBERS_REF.write().await;
                                object_ids.into_iter().for_each(|object_id| {
                                    match write_client_objects.get_mut(&object_id) {
                                        None => {
                                            // Skip
                                        }
                                        Some(subscribers) => {
                                            subscribers.remove(&id);
                                        }
                                    }
                                });
                            }

                            // 3 - Remove the client from the server memory
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
                            let topic_rq = match RQ_TopicRequest::try_from(buffer.as_ref()){
                                Ok(request) => {
                                    request
                                }
                                Err(err) => {
                                    log(Error, TopicHandler, format!("Failed to construct the topic request structure. Request:\n{:?} \n\nError:\n{}", buffer, err));
                                    return;
                                }
                            };
                            let result;

                            match topic_rq.action {
                                TopicsAction::SUBSCRIBE => {
                                    // 1 - check if topic exist, and if client is already sub
                                    let already_subscribed = {
                                        let topics_sub_read = TOPICS_SUBSCRIBERS_REF.read().await;
                                        match topics_sub_read.get(&topic_rq.topic_id){
                                            None => {
                                                // topic is not created so it must be not already sub
                                                false
                                            }
                                            Some(subscribers) => {
                                                // Topic exist : check if the client is is already known or not
                                                subscribers.contains(&client_id)
                                            }
                                        }
                                    };

                                    // this is false when the client is already sub
                                    if !already_subscribed {
                                        // 2 - inform the client that he's subscribed and update personal and global array
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
                                    // remove the topic_id from the personal and global set
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
                        // 0 - lock resources to add new subscription in one loop
                        {
                            let mut topics_guard = self.topics.write().await;
                            let mut topics_list_write = TOPICS_SUBSCRIBERS_REF.write().await;
                            topic_ids.iter().for_each(|topic_id|{
                                // 1 - add topics to the local hashsets
                                topics_guard.insert(*topic_id);

                                // 2 - add topics from global hashsets
                                topics_list_write.entry(*topic_id)
                                    .and_modify(|subscribers|{
                                        subscribers.insert(self.id);
                                    })
                                    .or_insert(HashSet::from([self.id]));
                            });
                        }
                    }
                    RemoveSubscribedTopic {
                        topic_ids
                    } => {
                        {// 0 - lock resources to add new subscription in one loop
                            let mut topics_guard = self.topics.write().await;
                            let mut topics_list_writ = TOPICS_SUBSCRIBERS_REF.write().await;

                            topic_ids.iter().for_each(|topic_id| {
                                // 1 - remove topic from local hashsets
                                topics_guard.remove(topic_id);

                                // 2 - remove the client of each topics from global hashsets
                                topics_list_writ.entry(*topic_id)
                                    .and_modify(|subscribers| {
                                        subscribers.remove(&self.id);
                                    });
                            });
                        }
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
                            let request = match RQ_ObjectRequest::try_from(buffer.as_ref()){
                                Ok(request) => {
                                    request
                                }
                                Err(err) => {
                                    log(Error, ObjectHandler, format!("Failed to construct the object structure. Request:\n{:?} \n\nError:\n{}", buffer, err));
                                    return;
                                }
                            };
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

                                    // subscribe the creator to this object
                                    match subscribe_client_to_object(
                                        new_object_id,
                                        client_id,
                                        client_sender.clone()
                                    ).await{
                                        Ok(_) => {
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
                                        }
                                        Err(err) => {
                                            let flag: Vec<u8> = vec![0,0,0,0,0,0,0,1]; // was a create request

                                            match send_datagram(
                                                server_socket.clone(),
                                                &RQ_ObjectRequest_NACK::new(vec_to_u8(flag), request.object_id, "Failed to create the object due to broker error.").as_bytes(),
                                                client_addr,
                                                client_sender,
                                            ).await {
                                                Ok(_) => {
                                                    log(Info, ObjectHandler, format!("Failed to create the object requested by {} due to broker error. Successfully notify the client. Error:\n{}", client_id, err));
                                                }
                                                Err(error) => {
                                                    log(Info, ObjectHandler, format!("Failed to create the object requested by {} due to broker error. And fail notify the client. Object Error:\n{}\n\nSend datagram error:\n{}", client_id, err, error));
                                                }
                                            }
                                            return; // Do not execute th following code
                                        }
                                    }
                                } // End create
                                // Update the object and update Client subscriptions
                                ObjectFlags::UPDATE => {
                                    // Check if id type is valid
                                    if !is_object_id_valid(request.object_id).await {
                                        let flag: Vec<u8> = vec![0,0,0,0,1,0,0,0]; // was a failed update request

                                        match send_datagram(
                                            server_socket.clone(),
                                            &RQ_ObjectRequest_NACK::new(vec_to_u8(flag), request.object_id, "Invalid Object identifier sent in a RQ_ObjectRequestUpdate. Bad format or unknown id.").as_bytes(),
                                            client_addr,
                                            client_sender.clone(),
                                        ).await {
                                            Ok(_) => {
                                                log(Info, ObjectHandler, format!("{} has sent an invalid object identifier (bad format or unknown id) in a RQ_ObjectRequestUpdate. id : {:b}", client_id, request.object_id));
                                            }
                                            Err(error) => {
                                                log(Info, ObjectHandler, format!("Failed to send NACK (invalid object identifier in a RQ_ObjectRequestUpdate, bad format or unknown id) to {}.\nError: {}", client_id, error));
                                            }
                                        }
                                        return;
                                    }
                                    // Object id is correct : get the difference from incoming topics and current topics
                                    let current_topics = {
                                        match OBJECTS_TOPICS_REF.read().await.get(&request.object_id) {
                                            None => {
                                                //return an empty hashset to ensure the rest of the method
                                                log(Warning, ObjectHandler, format!("Failed to find topics list of the object {}.", request.object_id));
                                                HashSet::default()
                                            }
                                            Some(topics) => {
                                                topics.clone()
                                            }
                                        }
                                    };

                                    let (added_topics, removed_topics) = diff_hashsets(&request.topics, &current_topics);

                                    // Update topics subscription of each client subscribed to this topic
                                    let clients_sub = {
                                        match OBJECT_SUBSCRIBERS_REF.read().await.get(&request.object_id) {
                                            None => {
                                                //return the client id to ensure he get an answer to his request
                                                log(Warning, ObjectHandler, format!("Failed to find subscribers list of the object {}.", request.object_id));
                                                HashSet::from([client_id])
                                            }
                                            Some(subscribers) => {
                                                subscribers.clone()
                                            }
                                        }
                                    };

                                    for subscriber_id in clients_sub {
                                        let client_addr = match get_client_addr(subscriber_id).await {
                                            Ok(value) => {value}
                                            Err(_) => {
                                                // Can't find client address : disconnect the client
                                                log(Error, ObjectHandler, format!("Can't find client address. Disconnect the client but can't send Shutdown error."));
                                                return;
                                            }
                                        };
                                        let client_sender = match get_client_sender(client_id).await {
                                            Ok(sender) => {sender}
                                            Err(_) => {
                                                // sender can't be found : send a shutdown for sync error
                                                match server_socket.send_to(
                                                    &RQ_Shutdown::new(SYNC_ERROR).as_bytes(),
                                                    client_addr
                                                ).await {
                                                    Err(err) => {
                                                        log(Error, ObjectHandler, format!("Failed to send connexion error to {}. The client has common hashset sync issue. Error:\n{}", client_id, err));
                                                    }
                                                    _ => {}
                                                }
                                                return;
                                            }
                                        };

                                        // Unsub removed topics
                                        if removed_topics.len() > 0{
                                            let cmd_unsub = RemoveSubscribedTopic {
                                                topic_ids: removed_topics.clone()
                                            };
                                            let client_sender_clone = client_sender.clone();
                                            tokio::spawn(async move {
                                                match client_sender_clone.send(cmd_unsub).await {
                                                    Ok(_) => {}
                                                    Err(_) => {
                                                        return;
                                                    }
                                                };
                                            });
                                        }

                                        // Sub added topics
                                        if added_topics.len() > 0 {
                                            let cmd_sub = AddSubscribedTopic {
                                                topic_ids: added_topics.clone()
                                            };
                                            let client_sender_clone = client_sender.clone();
                                            tokio::spawn(async move {
                                                match client_sender_clone.send(cmd_sub).await {
                                                    Ok(_) => {}
                                                    Err(_) => {
                                                        return;
                                                    }
                                                };
                                            });
                                        }


                                        // Client are now updated to the new version of the object : send them a notification
                                        let flag = vec![1,0,0,0,0,0,1,0]; // was a valid update request



                                        match send_datagram(
                                            server_socket.clone(),
                                            &RQ_ObjectRequestDefault_ACK::new(vec_to_u8(flag), request.object_id).as_bytes(),
                                            client_addr,
                                            client_sender
                                        ).await {
                                            Ok(_) => {
                                                log(Info, ObjectHandler, format!("Update notify sent to {} for the object_id : {:b}", subscriber_id, request.object_id));
                                            }
                                            Err(error) => {
                                                log(Info, ObjectHandler, format!("Failed to send update notify to {}.\nError: {}", subscriber_id, error));
                                            }
                                        }
                                    } // end for

                                } // End update
                                // Delete the object and Update Client subscriptions
                                ObjectFlags::DELETE => {
                                    // Check if id type is valid
                                    if !is_object_id_valid(request.object_id).await {
                                        let flag: Vec<u8> = vec![0,0,0,0,0,1,0,0]; // was a failed delete request

                                        match send_datagram(
                                            server_socket.clone(),
                                            &RQ_ObjectRequest_NACK::new(vec_to_u8(flag), request.object_id, "Invalid Object identifier sent in a RQ_ObjectRequestDelete. Bad format or unknown id.").as_bytes(),
                                            client_addr,
                                            client_sender.clone(),
                                        ).await {
                                            Ok(_) => {
                                                log(Info, ObjectHandler, format!("{} has sent an invalid object identifier (bad format or unknown id) in a RQ_ObjectRequestDelete. id : {:b}", client_id, request.object_id));
                                            }
                                            Err(error) => {
                                                log(Info, ObjectHandler, format!("Failed to send NACK (invalid object identifier in a RQ_ObjectRequestDelete, bad format or unknown id) to {}.\nError: {}", client_id, error));
                                            }
                                        }
                                        return;
                                    }
                                    // Object id is correct

                                    // find every subscribers of this object
                                    let clients_sub = {
                                        match OBJECT_SUBSCRIBERS_REF.read().await.get(&request.object_id) {
                                            None => {
                                                // If none found, return only the client to confirm the deletion
                                                log(Warning, ObjectHandler, format!("Failed to find subscribers of the object {}.", request.object_id));
                                                HashSet::from([client_id])
                                            }
                                            Some(subscribers) => {
                                                subscribers.clone()
                                            }
                                        }
                                    };


                                    // unsubscribe every client sub to this object
                                    for subscriber_id in clients_sub {
                                        let client_addr = match get_client_addr(client_id).await {
                                            Ok(value) => {value}
                                            Err(_) => {
                                                // Can't find client address : disconnect the client
                                                log(Error, ObjectHandler, format!("Can't find client address. Disconnect the client but can't send Shutdown error."));
                                                return;
                                            }
                                        };
                                        let client_sender = match get_client_sender(client_id).await {
                                            Ok(sender) => {sender}
                                            Err(_) => {
                                                // sender can't be found : send a shutdown for sync error
                                                match server_socket.send_to(
                                                    &RQ_Shutdown::new(SYNC_ERROR).as_bytes(),
                                                    client_addr
                                                ).await {
                                                    Err(err) => {
                                                        log(Error, ObjectHandler, format!("Failed to send connexion error to {}. The client has common hashset sync issue. Error:\n{}", client_id, err));
                                                    }
                                                    _ => {}
                                                }
                                                return;
                                            }
                                        };

                                        match unsubscribe_client_to_object(
                                            subscriber_id,
                                            request.object_id,
                                            client_sender.clone()
                                        ).await {
                                            Ok(_) => {
                                                // Client are now unsub, send them a notify or a delete ack
                                                let mut flag: Vec<u8> = Vec::with_capacity(8);
                                                let success_msg: String;
                                                if client_id == subscriber_id {
                                                    flag.extend([1,0,0,0,0,1,0,0]); // was a valid delete request (current client is the source of the deletion)
                                                    success_msg = format!("Object delete notify sent to {} for the object_id : {:b}", subscriber_id, request.object_id);
                                                }else{
                                                    flag.extend([1,0,0,0,1,0,0,0]); // was a valid unsubscribe request (current client is not the source of the delete)
                                                    success_msg = format!("Unsubscribe notify sent to {} for the object_id : {:b}", subscriber_id, request.object_id);
                                                }

                                                match send_datagram(
                                                    server_socket.clone(),
                                                    &RQ_ObjectRequestDefault_ACK::new(vec_to_u8(flag), request.object_id).as_bytes(),
                                                    client_addr,
                                                    client_sender
                                                ).await {
                                                    Ok(_) => {
                                                        log(Info, ObjectHandler, success_msg);
                                                    }
                                                    Err(error) => {
                                                        if client_id == subscriber_id {
                                                            log(Error, ObjectHandler, format!("Failed to send object delete notify to {}.\nError: {}", subscriber_id, error));
                                                        }else{
                                                            log(Error, ObjectHandler, format!("Failed to send unsubscribe notify to {}.\nError: {}", subscriber_id, error));
                                                        }
                                                    }
                                                }
                                            }
                                            Err(err) => {
                                                // Client are now unsub, send them a notify or a delete ack
                                                let mut flag: Vec<u8> = Vec::with_capacity(8);
                                                let failure_msg: String;
                                                if client_id == subscriber_id {
                                                    flag.extend([0,0,0,0,0,1,0,0]); // was a failed delete request (current client is the source of the deletion)
                                                    failure_msg = format!("{} has tried to delete object {} but it failed. Error:\n{}", subscriber_id, request.object_id, err);
                                                }else{
                                                    flag.extend([0,0,0,0,1,0,0,0]); // was a failed unsubscribe request (current client is not the source of the delete)
                                                    failure_msg = format!("{} has tried to delete object {} but it failed. Error:\n{}", client_id, request.object_id, err);
                                                }

                                                match send_datagram(
                                                    server_socket.clone(),
                                                    &RQ_ObjectRequest_NACK::new(vec_to_u8(flag), request.object_id, &*failure_msg).as_bytes(),
                                                    client_addr,
                                                    client_sender
                                                ).await {
                                                    Ok(_) => {
                                                        log(Error, ObjectHandler, failure_msg);
                                                    }
                                                    Err(error) => {
                                                        if client_id == subscriber_id {
                                                            log(Error, ObjectHandler, format!("Failed to send object delete failure notify to {}.\nRequest Error:\n{}\n\nSend datagrams error:\n{}", subscriber_id, err, error));
                                                        }else{
                                                            log(Error, ObjectHandler, format!("Failed to send unsubscribe failure notify to {}.\n\nRequest Error:\n{}\n\nSend datagrams error:\n{}", subscriber_id, err, error));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    } // end for

                                    // Clean the object :
                                    // Delete topics and then the object itself
                                    {
                                        let keys_to_remove = match OBJECTS_TOPICS_REF.read().await.get(&request.object_id) {
                                            None => {
                                                // return an empty hashset to ensure the normal flow of the end of the method
                                                HashSet::default()
                                            }
                                            Some(topics) => {
                                                topics.clone()
                                            }
                                        };
                                        let mut topics_sub_ref =TOPICS_SUBSCRIBERS_REF.write().await;
                                        keys_to_remove.iter().for_each(move |id|{
                                            topics_sub_ref.remove(id);
                                        });
                                    }
                                    {
                                        OBJECTS_TOPICS_REF.write().await.remove(&request.object_id);
                                    }
                                }// End delete
                                // Subscribe the client to all topics of this object
                                ObjectFlags::SUBSCRIBE => {
                                    // 1 - Check if object if is valid :
                                    if !is_object_id_valid(request.object_id).await {
                                        let flag: Vec<u8> = vec![0,0,0,1,0,0,0,0]; // was a failed subscribe request

                                        match send_datagram(
                                            server_socket.clone(),
                                            &RQ_ObjectRequest_NACK::new(vec_to_u8(flag), request.object_id, "Invalid Object identifier sent in a RQ_ObjectRequestSubscribe. Bad format or unknown id.").as_bytes(),
                                            client_addr,
                                            client_sender.clone(),
                                        ).await {
                                            Ok(_) => {
                                                log(Info, ObjectHandler, format!("{} has sent an invalid object identifier (bad format or unknown id) in a RQ_ObjectRequestSubscribe. id : {:b}", client_id, request.object_id));
                                            }
                                            Err(error) => {
                                                log(Info, ObjectHandler, format!("Failed to send NACK (invalid object identifier in a RQ_ObjectRequestSubscribe, bad format or unknown id) to {}.\nError: {}", client_id, error));
                                            }
                                        }
                                        return;
                                    }

                                    // 2 - id valid : subscribe the client
                                    match subscribe_client_to_object(
                                        request.object_id,
                                        client_id,
                                        client_sender.clone()
                                    ).await {
                                        Ok(_) => {
                                            // 3 - answer the request
                                            let flag: Vec<u8> = vec![1,0,0,1,0,0,0,0]; // was a success subscribe request
                                            match send_datagram(
                                                server_socket.clone(),
                                                &RQ_ObjectRequestDefault_ACK::new(vec_to_u8(flag), request.object_id).as_bytes(),
                                                client_addr,
                                                client_sender.clone(),
                                            ).await {
                                                Ok(_) => {
                                                    log(Info, ObjectHandler, format!("{} has successfully subscribe to the object of id : {:b}", client_id, request.object_id));
                                                }
                                                Err(error) => {
                                                    log(Info, ObjectHandler, format!("Failed to send ACK to a RQ_ObjectRequestSubscribe to {}.\nError: {}", client_id, error));
                                                }
                                            }
                                        }
                                        Err(err) => {
                                            let flag: Vec<u8> = vec![0,0,0,1,0,0,0,0]; // was a subscribe request

                                            match send_datagram(
                                                server_socket.clone(),
                                                &RQ_ObjectRequest_NACK::new(vec_to_u8(flag), request.object_id, "Failed to subscribe to the object due to broker error.").as_bytes(),
                                                client_addr,
                                                client_sender,
                                            ).await {
                                                Ok(_) => {
                                                    log(Info, ObjectHandler, format!("Failed to subscribe {} to the object requested due to broker error. Successfully notify the client. Error:\n{}", client_id, err));
                                                }
                                                Err(error) => {
                                                    log(Info, ObjectHandler, format!("Failed to subscribe {} to the object requested due to broker error. And fail notify the client. Object Error:\n{}\n\nSend datagram error:\n{}", client_id, err, error));
                                                }
                                            }
                                            return; // Do not execute th following code
                                        }
                                    }


                                } // End subscribe
                                // Unsubscribe the client to all topics of this object
                                ObjectFlags::UNSUBSCRIBE => {
                                    // 1 - Check if object if is valid :
                                    if !is_object_id_valid(request.object_id).await {
                                        let flag: Vec<u8> = vec![0,0,0,1,0,0,0,0]; // was a failed unsubscribe request

                                        match send_datagram(
                                            server_socket.clone(),
                                            &RQ_ObjectRequest_NACK::new(vec_to_u8(flag), request.object_id, "Invalid Object identifier sent in a RQ_ObjectRequestUnsubscribe. Bad format or unknown id.").as_bytes(),
                                            client_addr,
                                            client_sender.clone(),
                                        ).await {
                                            Ok(_) => {
                                                log(Info, ObjectHandler, format!("{} has sent an invalid object identifier (bad format or unknown id) in a RQ_ObjectRequestUnsubscribe. id : {:b}", client_id, request.object_id));
                                            }
                                            Err(error) => {
                                                log(Info, ObjectHandler, format!("Failed to send NACK (invalid object identifier in a RQ_ObjectRequestUnsubscribe, bad format or unknown id) to {}.\nError: {}", client_id, error));
                                            }
                                        }
                                        return;
                                    }

                                    // 2 - id valid : unsubscribe the client
                                    match unsubscribe_client_to_object(
                                        client_id,
                                        request.object_id,
                                        client_sender.clone()
                                    ).await {
                                        Ok(_) => {
                                            // Client is now unsub, send him a notify
                                            let flag: Vec<u8> = vec![1,0,0,1,0,0,0,0]; // was a success unsubscribe request
                                            match send_datagram(
                                                server_socket.clone(),
                                                &RQ_ObjectRequestDefault_ACK::new(vec_to_u8(flag), request.object_id).as_bytes(),
                                                client_addr,
                                                client_sender.clone(),
                                            ).await {
                                                Ok(_) => {
                                                    log(Info, ObjectHandler, format!("{} has successfully unsubscribe to the object of id : {:b}", client_id, request.object_id));
                                                }
                                                Err(error) => {
                                                    log(Info, ObjectHandler, format!("Failed to send ACK to a RQ_ObjectRequestUnsubscribe to {}.\nError: {}", client_id, error));
                                                }
                                            }
                                        }
                                        Err(err) => {
                                            // failed to unsub the client, notify it
                                            let flag: Vec<u8> = vec![0,0,0,1,0,0,0,0]; // was a success unsubscribe request
                                            match send_datagram(
                                                server_socket.clone(),
                                                &RQ_ObjectRequest_NACK::new(vec_to_u8(flag), request.object_id, &*err).as_bytes(),
                                                client_addr,
                                                client_sender.clone(),
                                            ).await {
                                                Ok(_) => {
                                                    log(Info, ObjectHandler, format!("{} has failed to unsubscribe to the object of id : {:b}.\nError:\n{}", client_id, request.object_id, err));
                                                }
                                                Err(error) => {
                                                    log(Info, ObjectHandler, format!("Failed to send NACK to a RQ_ObjectRequestUnsubscribe that has failed to {}.\nUnsubscribe Error:\n{}\n\nSend datagram error:\n{}", client_id, err, error));
                                                }
                                            }
                                        }
                                    }
                                } // End unsubscribe
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
                                } // end unknown
                            }
                        });
                    } // end handle object
                    AddSubscribedObject { object_id } => {
                        self.objects.write().await.insert(object_id);
                    }
                    RemoveSubscribedObject { object_id } => {
                        self.objects.write().await.remove(&object_id);
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
    let client_sender = match get_client_sender(id).await {
        Ok(sender) => {sender}
        Err(_) => {
            // sender can't be found : send a shutdown for sync error
            match server_sender.send_to(
                &RQ_Shutdown::new(SYNC_ERROR).as_bytes(),
                socket
            ).await {
                Err(err) => {
                    log(Error, HeartbeatChecker, format!("Failed to send connexion error to {}. The client has common hashset sync issue. Error:\n{}", id, err));
                }
                _ => {}
            }
            return;
        }
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