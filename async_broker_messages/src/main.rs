use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::string::String;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use local_ip_address::local_ip;
use tokio::{join, net::UdpSocket};
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::time::sleep;
use crate::client::Client;

use crate::client_lib::{ClientActions, now_ms};
use crate::client_lib::ClientActions::{HandleDisconnect, HandlePong, StartManagers, UpdateClientLastRequest};
use crate::config::Config;
use crate::config::LogLevel::*;
use crate::datagram::*;
use crate::server_lib::*;
use crate::server_lib::LogSource::*;
use crate::topic::Topic;

mod config;
mod datagram;
mod server_lib;
mod topic;
mod client;
mod client_lib;

#[tokio::main]
async fn main() {
    println!("[Server] Hi there !");
    let config: Arc<Config> = Arc::new(Config::new());
    log(Info, Other, format!("Config generation complete : \n{:#?}", config), config.clone());
    println!("The ip of the server is {}:{}", local_ip().unwrap(), config.port);

    /*===============================
         Init all server variable
     ==============================*/
    // Flag showing if the server is running or not
    #[allow(unused)] let mut b_running;

    // The socket used by the server to exchange datagrams with clients
    let socket = UdpSocket::bind(format!("{}:{}", "0.0.0.0", config.port.parse::<i16>().unwrap())).await.unwrap();
    let socket_ref = Arc::new(socket);

    // Root topic which can be subscribed by clients
    // Every topics have sub topic to them, you can go through each one like in a tree
    let root = RwLock::new(Topic::new(1, "/".to_string())); // default root topics is "/"
    let root_ref = Arc::new(root);

    // List of clients represented by their address and their ID
    let clients: RwLock<HashMap<u64, Sender<ClientActions>>> = RwLock::new(HashMap::default()); // <Client ID, Sender> -> the sender is used to sent command through the mpsc channels
    let clients_ref = Arc::new(clients);
    let clients_structs: Arc<RwLock<HashMap<u64, Arc<tokio::sync::Mutex<Client>>>>> = Arc::new(RwLock::new(HashMap::default())); // used only to keep struct alive

    // List of clients address
    let clients_address: RwLock<HashMap<u64, SocketAddr>> = RwLock::new(HashMap::default()); // <Client ID, address>
    let clients_address_ref = Arc::new(clients_address);

    // List of time reference for ping requests
    let pings_ref: Arc<Mutex<HashMap<u8, u128>>> = Arc::new(Mutex::new(HashMap::default())); // <Ping ID, reference time in ms>

    // List of clients subscribed topic
    let clients_topics_ref: Arc<RwLock<HashMap<u64, HashSet<u64>>>> = Arc::new(RwLock::new(HashMap::default())); // <Client ID, [TopicsID]>

    log(Info, Other, format!("Server variables successfully initialized"), config.clone());

    // =============================
    //    Spawning async functions
    // =============================
    b_running = Arc::from(true);
    let config_ref = config.clone();
    let datagram_handler = tokio::spawn(async move {
        datagrams_handler(
            socket_ref.clone(),
            clients_ref.clone(),
            clients_address_ref.clone(),
            root_ref.clone(),
            pings_ref.clone(),
            b_running.clone(),
            clients_topics_ref.clone(),
            clients_structs.clone(),
            config_ref.clone(),
        ).await;
    });

    log(Info, Other, format!("Server is running ..."), config.clone());

    #[allow(unused)]
        let res1 = join!(datagram_handler);

    b_running = Arc::from(false);
    log(Info, Other, format!("Server has stopped ... Running status : {}", b_running), config.clone());
}


/**
This method handle every incoming datagram in the broker
@param receiver Arc<UdpSocket> : An atomic reference of the UDP socket of the server
@param clients_ref Arc<RwLock<HashMap<u64, Sender<ClientActions>>> : An atomic reference of the clients HashMap. The map is protected by a rwLock to be thread safe
@param root_ref Arc<RwLock<TopicV2>> : An atomic reference of root topics, protected by a rwlock to be thread safe
@param pings_ref Arc<Mutex<HashMap<u64, u128>>> : An atomic reference of the pings time references, protected by a mutex to be thread safe
@param clients_ping_ref Arc<Mutex<HashMap<u64, u128>>> : An atomic reference of client's ping hashmap, protected by a rwLock to be thread safe
@param b_running Arc<bool> : An atomic reference of the server status to stop the "thread" if server is stopping
@param client_has_heartbeat_ref Arc<RwLock<HashMap<u64, bool>>> : An atomic reference of the clients_heartbeat status.

@return none
 */
async fn datagrams_handler(
    receiver: Arc<UdpSocket>,
    clients: Arc<RwLock<HashMap<u64, Sender<ClientActions>>>>,
    clients_addresses: Arc<RwLock<HashMap<u64, SocketAddr>>>,
    root: Arc<RwLock<Topic>>,
    pings: Arc<Mutex<HashMap<u8, u128>>>,
    b_running: Arc<bool>,
    clients_topics: Arc<RwLock<HashMap<u64, HashSet<u64>>>>,
    clients_structs: Arc<RwLock<HashMap<u64, Arc<tokio::sync::Mutex<Client>>>>>,
    config: Arc<Config>
) {
    log(Info, DatagramsHandler, format!("Datagrams Handler spawned"), config.clone());
    // infinite loop receiving listening for datagrams
    loop {
        // 1 - create an empty buffer of size 1024
        let mut buf = [0; 1024];

        // 2 - Wait for bytes reception
        match receiver.recv_from(&mut buf).await {
            // 3 - Once bytes are received, check for errors
            Ok((n, src)) => {
                // 4 - if OK match on the first byte (MESSAGE_TYPE)
                log(Info, DatagramsHandler, format!("Received {} bytes from {}", n, src), config.clone());

                // Get the client id source of the request
                let client_id_found = get_client_id(&src, clients_addresses.clone().read().await.to_owned()).await;
                let mut client_id = 0;
                // Do not handle the datagram if the source is not auth and is not trying to connect
                if (MessageType::from(buf[0]) != MessageType::CONNECT) && client_id_found.is_none()
                {
                    log(Warning, DatagramsHandler, format!("Datagrams dropped from {}. Error : Not connected and trying to {}.", src.ip(), display_message_type(MessageType::from(buf[0]))), config.clone());
                    continue;
                } else if !client_id_found.is_none() {
                    // The client is already connected
                    client_id = client_id_found.unwrap();

                    // update his last time he sent a request
                    let client_sender = {
                        let map = clients.read().await;
                        map.get(&client_id).unwrap().clone()
                    };
                    let cmd = UpdateClientLastRequest {time: now_ms()};
                    tokio::spawn(async move {
                        let _ = client_sender.send(cmd).await;
                    });
                }


                match MessageType::from(buf[0]) {
                    MessageType::CONNECT => {
                        // 4.1 - A user is trying to connect to the server
                        log(Info, DatagramsHandler, format!("{} is trying to connect", src.ip()), config.clone());
                        let (already_client, sender) = handle_connect(src, clients.clone(), receiver.clone(), clients_addresses.clone(),clients_structs.clone(), config.clone()).await;
                        let client_id = get_client_id(&src, clients_addresses.clone().read().await.to_owned()).await.unwrap();

                        // New client connected spawn a task to do some verification
                        if !already_client {
                            let cmd = StartManagers {
                                clients:  clients.clone(),
                                client_topics: clients_topics.clone(),
                                clients_addresses: clients_addresses.clone(),
                                clients_structs: clients_structs.clone(),
                                b_running: b_running.clone(),
                                server_sender: receiver.clone(),
                                pings: pings.clone(),
                            };

                            tokio::spawn(async move {
                               let _ = sender.send(cmd).await;
                            });
                        }
                    }
                    MessageType::DATA => {
                        // 4.2 - A user is trying to sent data to the server
                        log(Info, DatagramsHandler, format!("{} is trying to sent data", client_id), config.clone());

                        // Clone needed variable
                        let receiver_ref = receiver.clone();
                        let clients_addresses_ref = clients_addresses.clone();
                        let config_ref = config.clone();
                        let clients_topics_ref = clients_topics.clone();
                        let clients_ref = clients.clone();
                        let _ = tokio::spawn(async move {
                            handle_data(
                                receiver_ref,
                                buf,
                                client_id,
                                clients_ref,
                                clients_addresses_ref,
                                clients_topics_ref,
                                config_ref,
                            ).await;
                        });
                    }
                    MessageType::OPEN_STREAM => {
                        // 4.3 - A user is trying to open a new stream
                        log(Info, DatagramsHandler, format!("{} is trying to open a new stream", client_id), config.clone());
                    }
                    MessageType::SHUTDOWN => {
                        // 4.4 - A user is trying to shutdown the connexion with the server
                        log(Info, DatagramsHandler, format!("{} is trying to shutdown the connexion with the server", client_id), config.clone());

                        let cmd = HandleDisconnect {
                            client_topics: clients_topics.clone(),
                            clients_ref: clients.clone(),
                            clients_addresses: clients_addresses.clone(),
                            clients_structs: clients_structs.clone(),
                        };

                        let client_sender = {
                            let map = clients.read().await;
                            map.get(&client_id).unwrap().clone()
                        };
                        tokio::spawn(async move {
                            let _ = client_sender.send(cmd).await;
                        });
                    }
                    MessageType::HEARTBEAT => {
                        /*// 4.5 - A user is trying to sent an heartbeat
                        log(Info, DatagramsHandler, format!("{} is trying to sent an heartbeat", client_id), config.clone());
                        // check if client is still connected
                        if clients.read().await.contains_key(&client_id) {
                            // TODO: voir si c'est ok ou pas de pas check si la dernier rq est un HB ou pas.
                        }*/
                    }
                    MessageType::OBJECT_REQUEST => {
                        // 4.6 - A user is trying to request an object
                        log(Info, DatagramsHandler, format!("{} is trying to request an object", client_id), config.clone());
                    }
                    MessageType::TOPIC_REQUEST => {
                        // 4.7 - A user is trying to request a new topic
                        log(Info, DatagramsHandler, format!("{} is trying to request a new topic", client_id), config.clone());


                        // Clone needed variable
                        let receiver_ref = receiver.clone();
                        let config_ref = config.clone();
                        let root_ref = root.clone();
                        let clients_topics_ref = clients_topics.clone();
                        let client_sender = {
                            clients.read().await.get(&client_id).unwrap().clone()
                        };

                        let _ = tokio::spawn(async move {
                            topics_request_handler(
                                receiver_ref,
                                buf,
                                client_id,
                                src,
                                client_sender,
                                clients_topics_ref,
                                root_ref,
                                config_ref,
                            ).await;
                        });
                    }
                    MessageType::PONG => {
                        // 4.8 - A user is trying to answer a ping request
                        log(Info, DatagramsHandler, format!("{} is trying to answer a ping request", client_id), config.clone());
                        let time = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis(); // Current time in ms

                        // If the client is offline, don't compute the request
                        if !is_online(client_id, clients.clone()).await { continue; };

                        let cmd = HandlePong {
                            ping_id: buf[1],
                            current_time: time,
                            pings_ref: pings.clone(),
                        };
                        // send the command :
                        let clients_ref = clients.clone();
                        tokio::spawn(async move {
                            clients_ref.read().await.get(&client_id).unwrap().send(cmd).await.expect(&*format!("Failed to send the handle pong command to the client {}", client_id));
                        });
                    }
                    MessageType::TOPIC_REQUEST_ACK | MessageType::OBJECT_REQUEST_ACK | MessageType::CONNECT_ACK | MessageType::HEARTBEAT_REQUEST | MessageType::PING => {
                        // 4.9 - invalid datagrams for the server
                        log(Warning, DatagramsHandler, format!("{} has sent an invalid datagram.", client_id), config.clone());
                    }
                    MessageType::UNKNOWN => {
                        log(Warning, DatagramsHandler, format!("Received unknown packet from {}", src.ip()), config.clone());
                    }
                }
            }
            Err(e) => {
                log(Error, DatagramsHandler, format!("Error: {}", e), config.clone());
            }
        }
    }
}

async fn handle_data(
    sender: Arc<UdpSocket>,
    buffer: [u8; 1024],
    client_id: u64,
    clients: Arc<RwLock<HashMap<u64, Sender<ClientActions>>>>,
    clients_addresses: Arc<RwLock<HashMap<u64, SocketAddr>>>,
    clients_topics: Arc<RwLock<HashMap<u64, HashSet<u64>>>>,
    config: Arc<Config>,
) {
    let data_rq = RQ_Data::from(buffer.as_ref());
    let read_client_topics = clients_topics.read().await;

    if !read_client_topics.contains_key(&data_rq.topic_id) {
        log(Warning, DataHandler, format!("Topic {} doesn't exist", data_rq.topic_id), config.clone());
        return;
    }

    let mut interested_clients = read_client_topics.get(&data_rq.topic_id).unwrap().clone();
    interested_clients.remove(&client_id);
    for client in interested_clients {
        let client_sender = {
            let map = clients.read().await;
            map.get(&client).unwrap().clone()
        };
        let client_addr = *clients_addresses.read().await.get(&client).unwrap();
        let data = RQ_Data::new(data_rq.topic_id, data_rq.data.clone());
        let data = data.as_bytes();
        let result = sender.send_to(&data, client_addr).await.unwrap();
        tokio::spawn(async move {
            save_server_last_request_sent(client_sender, client_id).await;
        });
        log(Info, DataHandler, format!("Sent {} bytes to {}", result, client_addr), config.clone());
    }
}


/**
 */
async fn topics_request_handler(
    sender: Arc<UdpSocket>,
    buffer: [u8; 1024],
    client_id: u64,
    client_addr: SocketAddr,
    client_sender: Sender<ClientActions>,
    clients_topics: Arc<RwLock<HashMap<u64, HashSet<u64>>>>,
    root_ref: Arc<RwLock<Topic>>,
    config: Arc<Config>,
) {
    // 1 - Init local variables
    let topic_rq = RQ_TopicRequest::from(buffer.as_ref());

    let topic_path = String::from_utf8(topic_rq.payload).unwrap();

    let result;

    match topic_rq.action {
        TopicsAction::SUBSCRIBE => {
            let topic_result = create_topics(&topic_path, root_ref.clone()).await;

            match topic_result {
                Ok(topic_id) => {
                    clients_topics.write().await.entry(topic_id)
                        .and_modify(|vec| {
                            vec.insert(client_id);
                        })
                        .or_insert({
                            let mut tmp = HashSet::new();
                            tmp.insert(client_id);
                            tmp
                        });
                    result = sender.send_to(&RQ_TopicRequest_ACK::new(topic_id, TopicsResponse::SUCCESS_SUB).as_bytes(), client_addr).await;
                    tokio::spawn(async move {
                        save_server_last_request_sent(client_sender, client_id).await;
                    });
                }
                Err(_) => {
                    result = sender.send_to(&RQ_TopicRequest_NACK::new(TopicsResponse::FAILURE_SUB, "Subscribing to {} failed :(".to_string()).as_bytes(), client_addr).await;
                    tokio::spawn(async move {
                        save_server_last_request_sent(client_sender, client_id).await;
                    });
                }
            }
        }
        TopicsAction::UNSUBSCRIBE => {
            let topic_id = custom_string_hash(&topic_path);

            clients_topics.write().await.entry(topic_id).and_modify(|vec| { vec.retain(|e| *e != client_id) });

            result = sender.send_to(&RQ_TopicRequest_ACK::new(topic_id, TopicsResponse::SUCCESS_USUB).as_bytes(), client_addr).await;
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
            log(Info, TopicHandler, format!("Send {} bytes to {}", bytes, client_id), config.clone());
        }
        Err(error) => {
            log(Info, HeartbeatChecker, format!("Failed to send ACK to {}.\nError: {}", client_id, error), config.clone());
        }
    }
}