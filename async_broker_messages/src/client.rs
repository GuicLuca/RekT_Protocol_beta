// This document contain the client struct. It is where all client data
// are saved and should be computed.
// @author : GuicLuca (lucasguichard127@gmail.com)
// date : 22/03/2023

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use tokio::net::UdpSocket;
use tokio::sync::{Mutex, oneshot};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::sleep;

use crate::client_lib::{client_has_sent_life_sign, ClientActions, now_ms};
use crate::client_lib::ClientActions::*;
use crate::config::Config;
use crate::config::LogLevel::*;
use crate::datagram::*;
use crate::server_lib::*;
use crate::server_lib::LogSource::*;

#[derive(Debug)]
pub struct Client {
    // Identifiers
    pub id: u64,
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
     * @param id: u64 The server identifier of the client
     * @param socket: SocketAddr The socket of the client (used to sent data through network)
     */
    pub fn new(id: u64, socket: SocketAddr, receiver: Receiver<ClientActions>) -> Client {
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
     */
    pub async fn save_client_request_timestamp(&mut self, time: u128, config: Arc<Config>) {
        log(Info, ClientManager, format!("Last client request time updated for client {}", self.id), config.clone());
        self.last_request_from_client = time;
    }

    /**
     * This method save the current time as the last request sent from the server to this client
     */
    pub async fn save_server_request_timestamp(&mut self, time: u128, config: Arc<Config>) {
        // Get the writ access to the value
        log(Info, ClientManager, format!("Last server request time updated for client {}", self.id), config.clone());
        self.last_request_from_server = time;
    }

    /**
     * This is the main function that receive all data of the client
     * and which return result or process logic operation on the client
     * during his connection time.
     */
    pub async fn manager(&mut self, config: Arc<Config>)
    {
        log(Info, ClientManager, format!("Manager spawned for client {}", self.id), config.clone());

        // while client is alive ...
        loop {
            // Receive actions
            while let Some(cmd) = self.receiver.recv().await {
                match cmd {

                    // This command return the value of the requested key
                    Get { key, resp } => {
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

                    HandleData {
                        sender,
                        buffer,
                        clients,
                        clients_addresses,
                        clients_topics,
                        config,
                    } => {
                        let requests_counter_ref = self.requests_counter.clone();
                        let client_id = self.id;
                        let config_ref = config.clone();
                        tokio::spawn(async move {
                            // 1 - Check if the request is newer than the previous one
                            let data_rq = RQ_Data::from(buffer.as_ref());
                            {
                                let mut write_rq_counter_ref = requests_counter_ref.write().unwrap();
                                if write_rq_counter_ref.contains_key(&data_rq.topic_id) {
                                    if data_rq.sequence_number < *write_rq_counter_ref.get(&data_rq.topic_id).unwrap()
                                    && data_rq.sequence_number > 50 // this condition allow circle id (id 0..49 will override MAX_ID)
                                        {
                                        // Request is out-dated
                                        log(Warning, DataHandler, format!("Client {} sent a data with a sequence number lower than the last one", client_id), config_ref.clone());
                                        return;
                                    }else{
                                        // New request, update the last request id
                                        write_rq_counter_ref.insert(data_rq.topic_id, data_rq.sequence_number);
                                    }
                                }
                            }


                            let mut interested_clients = {
                                let read_client_topics = clients_topics.read().await;
                                if !read_client_topics.contains_key(&data_rq.topic_id) {
                                    log(Warning, DataHandler, format!("Topic {} doesn't exist", data_rq.topic_id), config_ref.clone());
                                    return;
                                }

                                read_client_topics.get(&data_rq.topic_id).unwrap().clone()
                            };


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
                                let result = sender.send_to(&data, client_addr).await.unwrap();
                                tokio::spawn(async move {
                                    save_server_last_request_sent(client_sender, client_id).await;
                                });
                                log(Info, DataHandler, format!("Sent {} bytes to {}", result, client_addr), config_ref.clone());
                            }
                        });
                    }

                    // This command compute the ping and save the result in the local client object
                    HandlePong { ping_id, current_time, pings_ref } => {
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
                        log(Info, ClientManager, format!("There is {}ms of ping between {} and the server", round_trip, self.id), config.clone());
                    }

                    // The two following command update last interaction in the client and server sens
                    UpdateServerLastRequest { time } => {
                        self.save_server_request_timestamp(time, config.clone()).await;
                    }

                    UpdateClientLastRequest { time } => {
                        self.save_client_request_timestamp(time, config.clone()).await;
                    }

                    // This command clear client data and stop client's task
                    HandleDisconnect {
                        topics_subscribers,
                        clients_ref,
                        clients_addresses,
                        clients_structs,
                        client_sender
                    } => {
                        /* need to remove from :
                            1 - client_topic : remove the ic from every subscribed
                            2 - clients_ref hashmap => will stop ping_sender task and heartbeat_checker task
                            3 - clients_addresses => this address is now free for a new connection
                        */
                        // 1 - loops trough client_topics topics and remove the client from the topic
                        let topic_ids: Vec<u64> = {
                            self.topics.read().unwrap().iter().cloned().collect() // transform the hashset to a vec
                        };

                        // last used of the channel is done : close it to cancel all pending requests
                        self.receiver.close();
                        let id = self.id;
                        let config_ref = config.clone();
                        tokio::spawn(async move{
                            {
                                let mut write_client_topics = topics_subscribers.write().await;
                                for topic_id in topic_ids {
                                    write_client_topics.get_mut(&topic_id).unwrap().remove(&id);
                                }
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
                            log(Info, Other, format!("Disconnection of client {}", id), config_ref.clone());
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
                        let config_ref = config.clone();
                        let server_sender_ref = server_sender.clone();
                        let id = self.id;
                        let socket = self.socket;
                        let clients_ref = clients.clone();
                        tokio::spawn(async move {
                            heartbeat_manager(id, socket, clients_ref, topics_subscribers, clients_addresses, clients_structs, b_running, server_sender_ref, config_ref).await;
                        });
                    }

                    // This command will compute the request and respond to the client
                    HandleTopicRequest {
                        server_socket,
                        topics_subscribers,
                        buffer,
                        root_ref,
                        client_sender,
                    } => {
                        let client_id = self.id;
                        let client_addr = self.socket;
                        let config_ref = config.clone();
                        tokio::spawn(async move {
                            // 1 - Init local variables
                            let topic_rq = RQ_TopicRequest::from(buffer.as_ref());
                            let topic_path = String::from_utf8(topic_rq.payload).unwrap();
                            let result;

                            match topic_rq.action {
                                TopicsAction::SUBSCRIBE => {
                                    let topic_result = create_topics(&topic_path, root_ref.clone()).await;

                                    match topic_result {
                                        Ok(topic_id) => {
                                            // Update topics_subscribers array
                                            {
                                                let mut topics_sub_write = topics_subscribers.write().await;
                                                let subscribers_set = topics_sub_write.entry(topic_id)
                                                    .or_insert_with(||HashSet::new());
                                                subscribers_set.insert(client_id);
                                            }
                                            // Add the new topic id to the client
                                            let cmd = AddSubscribedTopic { topic_id };
                                            let cmd_sender = client_sender.clone();
                                            tokio::spawn(async move {
                                                match cmd_sender.send(cmd).await {
                                                    Ok(_)=>{}
                                                    Err(_)=> {
                                                        return;
                                                    }
                                                };
                                            });

                                            result = server_socket.send_to(&RQ_TopicRequest_ACK::new(topic_id, TopicsResponse::SUCCESS_SUB).as_bytes(), client_addr).await;
                                            tokio::spawn(async move {
                                                save_server_last_request_sent(client_sender, client_id).await;
                                            });
                                        }
                                        Err(_) => {
                                            result = server_socket.send_to(&RQ_TopicRequest_NACK::new(TopicsResponse::FAILURE_SUB, "Subscribing to {} failed :(".to_string()).as_bytes(), client_addr).await;
                                            tokio::spawn(async move {
                                                save_server_last_request_sent(client_sender, client_id).await;
                                            });
                                        }
                                    }
                                }
                                TopicsAction::UNSUBSCRIBE => {
                                    // get the hash of the topic
                                    let topic_id = custom_string_hash(&topic_path);
                                    // Remove it from the common array
                                    {
                                        topics_subscribers.write().await.entry(topic_id).and_modify(|vec| { vec.retain(|e| *e != client_id) });
                                    }

                                    // remove it from the personal client set
                                    let cmd = RemoveSubscribedTopic { topic_id };
                                    let cmd_sender = client_sender.clone();
                                    tokio::spawn(async move {
                                        match cmd_sender.send(cmd).await{
                                            Ok(_)=>{}
                                            Err(_)=> {
                                                return;
                                            }
                                        };
                                    });

                                    result = server_socket.send_to(&RQ_TopicRequest_ACK::new(topic_id, TopicsResponse::SUCCESS_USUB).as_bytes(), client_addr).await;
                                    tokio::spawn(async move {
                                        save_server_last_request_sent(client_sender, client_id).await;
                                    });
                                }
                                TopicsAction::UNKNOWN => {
                                    result = Ok(0);
                                }
                            }

                            match result {
                                Ok(bytes) => {
                                    log(Info, TopicHandler, format!("Send {} bytes to {}", bytes, client_id), config_ref.clone());
                                }
                                Err(error) => {
                                    log(Info, HeartbeatChecker, format!("Failed to send ACK to {}.\nError: {}", client_id, error), config_ref.clone());
                                }
                            }
                        });
                    }
                    AddSubscribedTopic { topic_id } => {
                        self.topics.write().unwrap().insert(topic_id); // Warning: code bloquant
                    }
                    RemoveSubscribedTopic { topic_id } => {
                        self.topics.write().unwrap().remove(&topic_id); // Warning: code bloquant
                    }
                    GetTopics { resp } => {
                        let res = {
                            self.topics.read().unwrap().clone()
                        };
                        // Ignore errors
                        resp.send(Ok(res)).expect("Error while sending response to client");
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
 */
pub async fn heartbeat_manager(
    id: u64,
    socket: SocketAddr,
    clients: Arc<tokio::sync::RwLock<HashMap<u64, Sender<ClientActions>>>>,
    topics_subscribers: Arc<tokio::sync::RwLock<HashMap<u64, HashSet<u64>>>>,
    clients_addresses: Arc<tokio::sync::RwLock<HashMap<u64, SocketAddr>>>,
    clients_structs: Arc<tokio::sync::RwLock<HashMap<u64, Arc<Mutex<Client>>>>>,
    b_running: Arc<bool>,
    server_sender: Arc<UdpSocket>,
    config: Arc<Config>,
) {
    log(Info, HeartbeatChecker, format!("HeartbeatChecker spawned for {}", id), config.clone());
    // 1 - Init local variables
    let mut missed_heartbeats: u8 = 0; // used to count how many HB are missing
    let client_sender = {
        clients.read().await.get(&id).unwrap().clone()
    };

    // 2 - Loop while server is running and client is online
    while *b_running && is_online(id, clients.clone()).await {
        // 3 - waite for the heartbeat period
        sleep(Duration::from_secs(config.heart_beat_period as u64)).await;


        // 6 - check if client has sent heartbeat
        if !client_has_sent_life_sign(client_sender.clone(), config.clone()).await {
            // increase the missing packet
            missed_heartbeats += 1;
            log(Info, HeartbeatChecker, format!("{} hasn't sent life signs {} times", id, missed_heartbeats), config.clone());
            if missed_heartbeats == 2 {
                // 7 - send an heartbeat request
                let result = server_sender.send_to(&RQ_Heartbeat_Request::new().as_bytes(), socket).await;
                let sender_ref = client_sender.clone();
                tokio::spawn(async move {
                    save_server_last_request_sent(sender_ref, id).await;
                });
                match result {
                    Ok(bytes) => {
                        log(Info, HeartbeatChecker, format!("Send {} bytes (RQ_Heartbeat_Request) to {}", bytes, id), config.clone());
                    }
                    Err(error) => {
                        log(Error, HeartbeatChecker, format!("Failed to send RQ_HeartBeat_Request to {}. \nError: {}", id, error), config.clone());
                    }
                }
            } else if missed_heartbeats == 4 {
                // 8 - No Heartbeat for 4 period = end the client connection.
                // 8.1 - Send shutdown request
                let result = server_sender.send_to(&RQ_Shutdown::new(EndConnexionReason::SHUTDOWN).as_bytes(), socket).await;
                let sender_ref = client_sender.clone();
                tokio::spawn(async move {
                    save_server_last_request_sent(sender_ref, id).await;
                });
                match result {
                    Ok(bytes) => {
                        log(Info, HeartbeatChecker, format!("Send {} bytes (RQ_Heartbeat_Request) to {}", bytes, id), config.clone());
                    }
                    Err(error) => {
                        log(Error, HeartbeatChecker, format!("Failed to send RQ_Shutdown to {}. \nError: {}", id, error), config.clone());
                    }
                }

                // 8.2 - Remove the client from the main array
                // client's task will end automatically
                let sender_ref = client_sender.clone();
                let cmd = HandleDisconnect {
                    topics_subscribers: topics_subscribers.clone(),
                    clients_ref: clients.clone(),
                    clients_addresses: clients_addresses.clone(),
                    clients_structs: clients_structs.clone(),
                    client_sender: sender_ref,
                };
                let sender_ref = client_sender.clone();
                match sender_ref.send(cmd).await{
                    Ok(_)=>{}
                    Err(_)=> {
                        return;
                    }
                };
            }
        } else {
            // 7bis - reset flags variable
            missed_heartbeats = 0;
            let result = server_sender.send_to(&RQ_Heartbeat::new().as_bytes(), socket).await;
            let sender_ref = client_sender.clone();
            tokio::spawn(async move {
                save_server_last_request_sent(sender_ref, id).await;
            });
            match result {
                Ok(_) => {
                    log(Info, HeartbeatChecker, format!("Respond to client bytes (HeartBeat) to {}", id), config.clone());
                }
                Err(error) => {
                    log(Error, HeartbeatChecker, format!("Failed to send RQ_HeartBeat to {}.\nError: {}", id, error), config.clone());
                }
            }
        }
    }

    // 9 - End the task
    log(Info, HeartbeatChecker, format!("Heartbeat checker destroyed for {}", id), config.clone());
}