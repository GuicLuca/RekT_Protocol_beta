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

use tokio::sync::{Mutex};
use tokio::sync::mpsc::{Receiver};
use tokio::time::sleep;

use crate::client_lib::{client_has_sent_life_sign, ClientActions, now_ms};
use crate::client_lib::ClientActions::*;
use crate::config::Config;
use crate::config::LogLevel::*;
use crate::datagram::*;
use crate::server_lib::*;
use crate::server_lib::LogSource::*;
use crate::types::{ClientId, ClientSender, ClientsHashMap, ServerSocket, TopicsHashMap};

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
     * @param id: ClientId The server identifier of the client
     * @param socket: SocketAddr The socket of the client (used to sent data through network)
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
                                               config: &Arc<Config>,
    ) {
        log(Info, ClientManager, format!("Last client request time updated for client {}", self.id), &config);
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
                                               config: &Arc<Config>,
    ) {
        log(Info, ClientManager, format!("Last server request time updated for client {}", self.id), &config);
        self.last_request_from_server = time;
    }

    /**
     * This is the main function that receive all data of the client
     * and which return result or process logic operation on the client
     * while the connection is up.
     *
     * @param config: &Arc<Config>
     */
    pub async fn manager(&mut self,
                         config: &'static Arc<Config>,
    )
    {
        log(Info, ClientManager, format!("Manager spawned for client {}", self.id), &config);

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
                        clients,
                        clients_addresses,
                        topics_subscribers,
                        config,
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
                                        log(Warning, DataHandler, format!("Client {} sent a data with a sequence number lower than the last one", client_id), &config);
                                        return;
                                    }
                                }

                                // New request, update the last request id / or insert it if the topic_id is unknown
                                write_rq_counter_ref.insert(data_rq.topic_id, data_rq.sequence_number);
                                log(Info, DataHandler, format!("New data sequence for client {}.", client_id), &config);
                            }

                            // 3 - get a list of interested clients
                            let mut interested_clients = {
                                let read_client_topics = topics_subscribers.read().await;
                                if !read_client_topics.contains_key(&data_rq.topic_id) {
                                    log(Warning, DataHandler, format!("Topic {} doesn't exist", data_rq.topic_id), &config);
                                    return;
                                }

                                read_client_topics.get(&data_rq.topic_id).unwrap().clone()
                            };

                            // 4 - remove the source client and then sent the new data to every interested client
                            interested_clients.remove(&client_id);
                            for client in interested_clients {
                                let client_sender = {
                                    let map = clients.read().await;
                                    map.get(&client).unwrap().clone()
                                };
                                let data = RQ_Data::new(data_rq.sequence_number, data_rq.topic_id, data_rq.data.clone());
                                let data = data.as_bytes();
                                let client_addr = {
                                    // use closure to reduce the lock lifetime
                                    *clients_addresses.read().await.get(&client).unwrap()
                                };
                                let result = send_datagram(sender.clone(), &data, client_addr, client_sender).await;

                                log(Info, DataHandler, format!("Data propagation : Sent {} bytes to {}", result.unwrap(), client_addr), &config);
                            }
                        });
                    }

                    // This command compute the ping and save the result in the local client object
                    HandlePong {
                        ping_id,
                        current_time,
                        pings_ref
                    } => {
                        let round_trip;
                        {
                            // 1 - get the mutable ref of all ping request
                            let mut pings_ref_mut = pings_ref.lock().await;
                            // 2 - compute the round trip
                            round_trip = (current_time - pings_ref_mut.get(&ping_id).unwrap()) / 2;
                            // 3 - free the ping_id
                            pings_ref_mut.remove(&ping_id);
                        }
                        // 4 - set the ping for the client
                        self.ping = round_trip;
                        log(Info, ClientManager, format!("There is {}ms of ping between {} and the server", round_trip, self.id), &config);
                    }

                    // The two following command update last interaction in the client and server sens
                    UpdateServerLastRequest { time } => {
                        self.save_server_request_timestamp(time, &config).await;
                    }
                    UpdateClientLastRequest { time } => {
                        self.save_client_request_timestamp(time, &config).await;
                    }

                    // This command clear client data and stop client's task
                    HandleDisconnect {
                        topics_subscribers,
                        clients_ref,
                        clients_addresses,
                        clients_structs
                    } => {
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
                                let mut write_client_topics = topics_subscribers.write().await;
                                topic_ids.into_iter().for_each(|topic_id| {
                                    write_client_topics.get_mut(&topic_id).unwrap().remove(&id);
                                });
                            }

                            // 2&3 - Remove the id from the server memory
                            {
                                clients_ref.write().await.remove(&id);
                            }
                            {
                                clients_addresses.write().await.remove(&id);
                            }
                            {
                                clients_structs.write().await.remove(&id);
                            }
                            log(Info, Other, format!("Disconnection of client {}", id), &config);
                        });
                    }

                    // This command start every client managers
                    StartManagers {
                        clients,
                        topics_subscribers,
                        clients_addresses,
                        clients_structs,
                        b_running,
                        server_sender,
                    } => {

                        // heartbeat_manager :
                        let server_sender_ref = server_sender.clone();
                        let id = self.id;
                        let socket = self.socket;
                        let clients_ref = clients.clone();
                        tokio::spawn(async move {
                            heartbeat_manager(id, socket, clients_ref, topics_subscribers, clients_addresses, clients_structs, b_running, server_sender_ref, &config).await;
                        });
                    }

                    // This command will compute a topic request, update the client
                    // and then answer the client with a ACK or NACK.
                    HandleTopicRequest {
                        server_socket,
                        topics_subscribers,
                        buffer,
                        client_sender,
                    } => {
                        let client_id = self.id;
                        let client_addr = self.socket;
                        tokio::spawn(async move {
                            // 1 - Init local variables
                            let topic_rq = RQ_TopicRequest::from(buffer.as_ref());
                            let topic_path = String::from_utf8(topic_rq.payload).unwrap();
                            let topic_hash = custom_string_hash(&topic_path);
                            let result;

                            match topic_rq.action {
                                TopicsAction::SUBSCRIBE => {
                                    // 1 - check if topic exist, then insert/update the value by adding the client id to the topic
                                    let is_subscribed = {
                                        let mut topics_sub_write = topics_subscribers.write().await;
                                        let subscribers_set = topics_sub_write.entry(topic_hash)
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
                                        let cmd = AddSubscribedTopic { topic_id: topic_hash };
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
                                            &RQ_TopicRequest_ACK::new(topic_hash, TopicsResponse::SUCCESS_SUB).as_bytes(),
                                            client_addr,
                                            client_sender,
                                        ).await;

                                        match result {
                                            Ok(_) => {
                                                log(Info, TopicHandler, format!("{} has successfully sub the topic {}", client_id, topic_hash), &config);
                                            }
                                            Err(error) => {
                                                log(Info, HeartbeatChecker, format!("Failed to send ACK to {}.\nError: {}", client_id, error), &config);
                                            }
                                        }
                                    } else {
                                        // 2bis - Respond with an error message
                                        result = send_datagram(
                                            server_socket.clone(),
                                            &RQ_TopicRequest_NACK::new(TopicsResponse::FAILURE_SUB, "Already subscribed to {}.".to_string()).as_bytes(),
                                            client_addr,
                                            client_sender,
                                        ).await;

                                        match result {
                                            Ok(_) => {
                                                log(Info, TopicHandler, format!("{} has failed sub the topic {} (ALREADY SUB)", client_id, topic_hash), &config);
                                            }
                                            Err(error) => {
                                                log(Info, HeartbeatChecker, format!("Failed to send ACK to {}.\nError: {}", client_id, error), &config);
                                            }
                                        }
                                    }
                                }
                                TopicsAction::UNSUBSCRIBE => {
                                    // get the hash of the topic
                                    // Remove it from the common array
                                    {
                                        topics_subscribers.write().await.entry(topic_hash).and_modify(|vec| { vec.retain(|e| *e != client_id) });
                                    }

                                    // remove it from the personal client set
                                    let cmd = RemoveSubscribedTopic { topic_id: topic_hash };
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
                                        &RQ_TopicRequest_ACK::new(topic_hash, TopicsResponse::SUCCESS_USUB).as_bytes(),
                                        client_addr,
                                        client_sender,
                                    ).await;

                                    match result {
                                        Ok(_) => {
                                            log(Info, TopicHandler, format!("{} has successfully unsub the topic {}", client_id, topic_hash), &config);
                                        }
                                        Err(error) => {
                                            log(Info, HeartbeatChecker, format!("Failed to send ACK to {}.\nError: {}", client_id, error), &config);
                                        }
                                    }
                                }
                                TopicsAction::UNKNOWN => {}
                            }
                        });
                    }

                    // The two following task update the subscribed topics of this client
                    AddSubscribedTopic {
                        topic_id
                    } => {
                        self.topics.write().unwrap().insert(topic_id); // Warning: code bloquant
                    }
                    RemoveSubscribedTopic {
                        topic_id
                    } => {
                        self.topics.write().unwrap().remove(&topic_id); // Warning: code bloquant
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
 * @param clients: ClientsHashMap<ClientSender>, HashMap containing every client channel. Used by the disconnect method.
 * @param topics_subscriber: TopicsHashMap<HashSet<ClientId>>, HashMap containing every client Id that are subscribed to a topic. Used by the disconnect method.
 * @param clients_addresses: ClientsHashMap<SocketAddr>, HashMap containing every client address. Used by the disconnect method.
 * @param clients_structs: ClientsHashMap<Arc<Mutex<Client>>>, HashMap containing every client struct. Used by the disconnect method.
 * @param b_running: Arc<bool>, State of the server
 * @param server_sender: ServerSocket, the server socket used to exchange data
 * @param config: &Arc<Config>, The config reference used to access to the heartbeat period
 */
pub async fn heartbeat_manager(
    id: ClientId,
    socket: SocketAddr,
    clients: ClientsHashMap<ClientSender>,
    topics_subscribers: TopicsHashMap<HashSet<ClientId>>,
    clients_addresses: ClientsHashMap<SocketAddr>,
    clients_structs: ClientsHashMap<Arc<Mutex<Client>>>,
    b_running: Arc<bool>,
    server_sender: ServerSocket,
    config: &Arc<Config>,
) {
    log(Info, HeartbeatChecker, format!("HeartbeatChecker spawned for {}", id), &config);
    // 1 - Init local variables
    let mut missed_heartbeats: u8 = 0; // used to count how many HB are missing
    let client_sender = {
        clients.read().await.get(&id).unwrap().clone()
    };

    // 2 - Loop while server is running and client is online
    while *b_running && is_online(id, clients.clone()).await {
        // 3 - waite for the heartbeat period
        sleep(Duration::from_secs(config.heart_beat_period as u64)).await;


        // 4 - check if client has sent heartbeat
        if !client_has_sent_life_sign(client_sender.clone(), &config).await {
            //  5 - increase the missing packet
            missed_heartbeats += 1;
            log(Info, HeartbeatChecker, format!("{} hasn't sent life signs {} times", id, missed_heartbeats), &config);
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
                        log(Info, HeartbeatChecker, format!("Send {} bytes (RQ_Heartbeat_Request) to {}", bytes, id), &config);
                    }
                    Err(error) => {
                        log(Error, HeartbeatChecker, format!("Failed to send RQ_HeartBeat_Request to {}. \nError: {}", id, error), &config);
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
                        log(Info, HeartbeatChecker, format!("Send {} bytes (RQ_Heartbeat_Request) to {}", bytes, id), &config);
                    }
                    Err(error) => {
                        log(Error, HeartbeatChecker, format!("Failed to send RQ_Shutdown to {}. \nError: {}", id, error), &config);
                    }
                }

                // 8 - Remove the client from the main array
                // client's task will end automatically
                let cmd = HandleDisconnect {
                    topics_subscribers: topics_subscribers.clone(),
                    clients_ref: clients.clone(),
                    clients_addresses: clients_addresses.clone(),
                    clients_structs: clients_structs.clone(),
                };
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
                    log(Info, HeartbeatChecker, format!("Respond to client bytes (HeartBeat) to {}", id), &config);
                }
                Err(error) => {
                    log(Error, HeartbeatChecker, format!("Failed to send RQ_HeartBeat to {}.\nError: {}", id, error), &config);
                }
            }
        }
    }

    // 9 - End the task
    log(Info, HeartbeatChecker, format!("Heartbeat checker destroyed for {}", id), &config);
}