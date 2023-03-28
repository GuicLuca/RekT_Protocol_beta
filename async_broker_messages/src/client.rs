// This document contain the client struct. It is where all client data
// are saved and should be computed.
// @author : GuicLuca (lucasguichard127@gmail.com)
// date : 22/03/2023

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use bytes::Bytes;

use tokio::net::UdpSocket;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::client_lib::{client_has_sent_life_sign, ClientActions, now_ms};
use crate::client_lib::ClientActions::HandleDisconnect;
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
    last_request_from_client: RwLock<u128>,
    last_request_from_server: RwLock<u128>,
    // Timestamp of the last request received from this user
    missed_heartbeats: u8,
    // amount of heartbeats_request no received
    ping: u128, // client ping in ms

    // Topics
    topics: RwLock<Vec<u64>>,
    // Contain each subscribed topic id
    requests_counter: RwLock<HashMap<u64, u8>>, // Contain the id of the last request on a topic for scheduling
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
            last_request_from_client: RwLock::from(now_ms()),
            last_request_from_server: RwLock::from(0),
            missed_heartbeats: 0,
            ping: 0,
            topics: RwLock::from(Vec::new()),
            requests_counter: RwLock::from(HashMap::new()),
        }
    }

    /**
     * This method save the current time as the last client request received
     */
    pub fn save_client_request_timestamp(&mut self, time: u128, config: Arc<Config>) {
        // Get the writ access to the value
        let config = Arc::new(Config::new());
        log(Info, ClientManager, format!("Last client request time updated for client {}", self.id), config.clone());

        let mut last_request_mut = self.last_request_from_client.write().unwrap();
        // Update it at now
        *last_request_mut = time;
    }

    /**
     * This method save the current time as the last request sent from the server to this client
     */
    pub fn save_server_request_timestamp(&mut self, time: u128, config: Arc<Config>) {
        // Get the writ access to the value
        let config = Arc::new(Config::new());
        log(Info, ClientManager, format!("Last server request time updated for client {}", self.id), config.clone());
        let mut last_request_mut = self.last_request_from_server.write().unwrap();
        // Update it at now
        *last_request_mut = time;
    }

    /**
     * This is the main function that receive all data of the client
     * and which return result or process logic operation on the client
     * during his connection time.
     */
    pub async fn manager(&mut self, config:Arc<Config>)
    {
        log(Info, ClientManager, format!("Manager spawned for client {}", self.id), config.clone());

        // while client is alive ...
        loop {
            // Receive actions
            while let Some(cmd) = self.receiver.recv().await {
                match cmd {

                    // This command return the value of the requested key
                    ClientActions::Get { key, resp } => {
                        match key.as_str() {
                            // list of valid key under :
                            "id" => {
                                let res = self.id;
                                // Ignore errors
                                let _ = resp.send(Ok(res as u128));
                            },
                            "last_request_from_client" => {
                                let res = *self.last_request_from_client.read().unwrap();
                                // Ignore errors
                                let _ = resp.send(Ok(res));
                            }
                            _ => {
                                let _ = resp.send(Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Bad key requested : \"{}\"", key))));
                            }
                        }
                    }

                    // This command compute the ping and save the result in the local client object
                    ClientActions::HandlePong { ping_id, current_time, pings_ref } => {
                        // 1 - get the mutable ref of all ping request
                        let mut pings_ref_mut = pings_ref.lock().await;
                        // 2 - compute the round trip
                        let round_trip = (current_time - pings_ref_mut.get(&ping_id).unwrap()) / 2;
                        // 3 - free the ping_id
                        pings_ref_mut.remove(&ping_id);
                        // 4 - set the ping for the client
                        self.ping = round_trip;
                        log(Info, ClientManager, format!("There is {}ms of ping between {} and the server", round_trip, self.id), config.clone());
                    }

                    // The two following command update last interaction in the client and server sens
                    ClientActions::UpdateServerLastRequest { time} => {
                        self.save_server_request_timestamp(time, config.clone());
                    }

                    ClientActions::UpdateClientLastRequest { time } => {
                        self.save_client_request_timestamp(time, config.clone());
                    }

                    // This command clear client data and stop client's task
                    ClientActions::HandleDisconnect {
                        client_topics,
                        clients_ref,
                        clients_addresses,
                        clients_structs
                    } => {
                        let client_id = self.id.clone();
                        let config_ref = config.clone();
                        tokio::spawn(async move {
                            /* need to remove from :
                                1 - client_topic : remove the ic from every subscribed
                                2 - clients_ref hashmap => will stop ping_sender task and heartbeat_checker task
                                3 - clients_addresses => this address is now free for a new connection
                            */
                            // 1 - loops trough client_topics topics and remove the client from the topic
                            let client_topics = &client_topics;
                            let topic_ids: Vec<u64> = {
                                let read_client_topics = client_topics.read().await;
                                read_client_topics.keys().cloned().collect()
                            };
                            let mut write_client_topics = client_topics.write().await;
                            for topic_id in topic_ids {
                                write_client_topics.get_mut(&topic_id).unwrap().remove(&client_id);
                            }

                            // 2&3 - Remove the id from the server memory
                            {
                                clients_ref.write().await.remove(&client_id);
                            }
                            {
                                clients_addresses.write().await.remove(&client_id);
                            }
                            {
                                clients_structs.write().await.remove(&client_id);
                            }
                            log(Info, Other, format!("Disconnection of client {}", client_id), config_ref);
                        });
                    },

                    // This command start every client managers
                    ClientActions::StartManagers {
                        clients,
                        client_topics,
                        clients_addresses,
                        clients_structs,
                        b_running,
                        server_sender,
                        pings,
                    } => {

                        // heartbeat_manager :
                        let config_ref = config.clone();
                        let server_sender_ref = server_sender.clone();
                        let id = self.id;
                        let socket = self.socket;
                        let clients_ref = clients.clone();
                        let b_running_ref = b_running.clone();
                        tokio::spawn(async move {
                            heartbeat_manager(id, socket, clients_ref, client_topics, clients_addresses, clients_structs, b_running, server_sender_ref, config_ref).await;
                        });

                        // ping_manager
                        let config_ref = config.clone();
                        let server_sender_ref = server_sender.clone();
                        let id = self.id;
                        let socket = self.socket;
                        let clients_ref = clients.clone();
                        let pings_ref = pings.clone();
                        let _ = tokio::spawn(async move {
                            ping_manager(
                                server_sender_ref,
                                id,
                                socket,
                                clients_ref,
                                pings_ref,
                                b_running_ref,
                                config_ref,
                            ).await;
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
 */
pub async fn heartbeat_manager(
    id: u64,
    socket: SocketAddr,
    clients: Arc<tokio::sync::RwLock<HashMap<u64, Sender<ClientActions>>>>,
    client_topics: Arc<tokio::sync::RwLock<HashMap<u64, HashSet<u64>>>>,
    clients_addresses: Arc<tokio::sync::RwLock<HashMap<u64, SocketAddr>>>,
    clients_structs: Arc<tokio::sync::RwLock<HashMap<u64, Arc<tokio::sync::Mutex<Client>>>>>,
    b_running: Arc<bool>,
    server_sender: Arc<UdpSocket>,
    config: Arc<Config>,
) {

    log(Info, HeartbeatChecker, format!("HeartbeatChecker sender spawned for {}", id), config.clone());
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
                let cmd = HandleDisconnect {
                    client_topics: client_topics.clone(),
                    clients_ref: clients.clone(),
                    clients_addresses: clients_addresses.clone(),
                    clients_structs: clients_structs.clone(),
                };
                let sender_ref = client_sender.clone();
                tokio::spawn(async move {
                    let _ = sender_ref.send(cmd).await;
                });
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



/** TODO : doc
This method send ping request to every connected clients
@param sender Arc<UdpSocket> : An atomic reference of the UDP socket of the server
@param client_id u64 : The client identifier.
@param clients: Arc<RwLock<HashMap<u64, SocketAddr>>> : An atomic reference of the pings HashMap. The map is protected by a rwlock to be thread safe
@param pings Arc<Mutex<HashMap<u8, u128>>> : An atomic reference of the pings HashMap. The map is protected by a mutex to be thread safe
@param b_running Arc<bool> : An atomic reference of the server status to stop the "thread" if server is stopping
@param clients_ping_ref Arc<Mutex<HashMap<u64, u128>>> : An atomic reference of client's ping hashmap, protected by a mutex to be thread safe

@return None
 */
async fn ping_manager(
    sender: Arc<UdpSocket>,
    client_id: u64,
    client_addr: SocketAddr,
    clients: Arc<tokio::sync::RwLock<HashMap<u64, Sender<ClientActions>>>>,
    pings: Arc<Mutex<HashMap<u8, u128>>>,
    b_running: Arc<bool>,
    config: Arc<Config>,
) {
    log(Info, PingSender, format!("Ping sender spawned for {}", client_id), config.clone());
    let client_sender = {
        let map = clients.read().await;
        map.get(&client_id).unwrap().clone()
    };


    // 1 - Loop while server is running and client is online
    while *b_running && is_online(client_id, clients.clone()).await {
        // 2 - Send a ping request to the client
        let result = sender.send_to(&RQ_Ping::new(get_new_ping_reference(pings.clone(), config.clone()).await).as_bytes(), client_addr).await;

        let sender_ref = client_sender.clone();
        tokio::spawn(async move {
            save_server_last_request_sent(sender_ref, client_id).await;
        });
        match result {
            Ok(bytes) => {
                log(Info, PingSender, format!("Send {} bytes to {}", bytes, client_id), config.clone());
            }
            Err(error) => {
                log(Error, PingSender, format!("Failed to send ping request to {}.\n{}", client_id, error), config.clone());
            }
        }
        // 3 - wait 10 sec before ping everyone again
        sleep(Duration::from_secs(config.ping_period as u64)).await;
    }
    // 4 - End the task
    log(Info, PingSender, format!("Ping sender destroyed for {}", client_id), config.clone());
}