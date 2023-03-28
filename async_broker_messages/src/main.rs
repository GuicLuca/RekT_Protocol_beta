use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use local_ip_address::local_ip;
use tokio::{join, net::UdpSocket};
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, Mutex};
use tokio::sync::RwLock;
use tokio::time::sleep;

use crate::client::Client;
use crate::client_lib::{ClientActions, now_ms};
use crate::client_lib::ClientActions::{HandleDisconnect, HandlePong, HandleTopicRequest, StartManagers, UpdateClientLastRequest};
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
    let clients_structs: Arc<RwLock<HashMap<u64, Arc<Mutex<Client>>>>> = Arc::new(RwLock::new(HashMap::default())); // used only to keep struct alive

    // List of clients address
    let clients_address: RwLock<HashMap<u64, SocketAddr>> = RwLock::new(HashMap::default()); // <Client ID, address>
    let clients_address_ref = Arc::new(clients_address);

    // List of time reference for ping requests
    let pings_ref: Arc<Mutex<HashMap<u8, u128>>> = Arc::new(Mutex::new(HashMap::default())); // <Ping ID, reference time in ms>

    // List of topic subscribers
    let topics_subscribers_ref: Arc<RwLock<HashMap<u64, HashSet<u64>>>> = Arc::new(RwLock::new(HashMap::default())); // <Topic ID, [Clients ID]>

    log(Info, Other, format!("Server variables successfully initialized"), config.clone());

    // =============================
    //    Spawning async functions
    // =============================
    b_running = Arc::from(true);
    let config_ref = config.clone();

    // clone needed value for the first task
    let datagram_socket = socket_ref.clone();
    let datagram_clients = clients_ref.clone();
    let datagram_clients_address = clients_address_ref.clone();
    let datagram_pings = pings_ref.clone();
    let datagram_b_running = b_running.clone();
    let datagram_config = config_ref.clone();

    let ping_sender = tokio::spawn(async move {
        ping_sender(
            socket_ref.clone(),
            clients_address_ref.clone(),
            clients_ref.clone(),
            pings_ref.clone(),
            b_running.clone(),
            config_ref.clone(),
        ).await;
    });
    // ... and then spawn the second task
    let datagram_handler = tokio::spawn(async move {
        datagrams_handler(
            datagram_socket,
            datagram_clients,
            datagram_clients_address,
            root_ref.clone(),
            datagram_pings,
            datagram_b_running,
            topics_subscribers_ref.clone(),
            clients_structs.clone(),
            datagram_config,
        ).await;
    });


    log(Info, Other, format!("Server is running ..."), config.clone());


    join!(datagram_handler, ping_sender);

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
    topics_subscribers: Arc<RwLock<HashMap<u64, HashSet<u64>>>>,
    clients_structs: Arc<RwLock<HashMap<u64, Arc<Mutex<Client>>>>>,
    config: Arc<Config>,
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
                    let cmd = UpdateClientLastRequest { time: now_ms() };
                    tokio::spawn(async move {
                        client_sender.send(cmd).await;
                    });
                }


                match MessageType::from(buf[0]) {
                    MessageType::CONNECT => {
                        // 4.1 - A user is trying to connect to the server
                        log(Info, DatagramsHandler, format!("{} is trying to connect", src.ip()), config.clone());

                        let b_running_ref = b_running.clone();
                        let socket_ref = receiver.clone();
                        let clients_addresses_ref = clients_addresses.clone();
                        let config_ref = config.clone();
                        let topics_subscribers_ref = topics_subscribers.clone();
                        let clients_structs_ref = clients_structs.clone();
                        let clients_ref = clients.clone();
                        let pings_ref = pings.clone();
                        tokio::spawn(async move{
                            let (is_connected, current_id) = already_connected(&src.ip(), clients_addresses_ref.read().await).await;
                            let uuid;
                            let result;

                            if is_connected {
                                // 2 - User already connected, return the same data as previous connection
                                uuid = current_id;
                                log(Info, DatagramsHandler, format!("{} was already a client, UUID : {}", src.ip(), uuid), config_ref.clone());
                            } else {
                                //3 - it's a new connection : generate a new id
                                uuid = get_new_id();
                                log(Info, DatagramsHandler, format!("{} is now a client, UUID : {}", src.ip(), uuid), config_ref.clone());

                                // 3.1 - Create a chanel to exchange commands through
                                let (sender, receiver) = mpsc::channel::<ClientActions>(32);
                                let new_client = Client::new(uuid, src, receiver);
                                let new_client_arc: Arc<Mutex<Client>> = Arc::new(tokio::sync::Mutex::new(new_client)); // clone new_client and wrap in Arc and Mutex

                                // 3.2 - fill server arrays to keep in memory the connection
                                {
                                    let mut map = clients_ref.write().await;
                                    map.insert(uuid, sender);
                                }
                                {
                                    let manager_ref = new_client_arc.clone();
                                    let config_ref_task = config_ref.clone();
                                    tokio::spawn(async move{
                                        manager_ref.lock().await.manager(config_ref_task).await;
                                    });
                                    let mut map = clients_structs_ref.write().await;
                                    map.insert(uuid, new_client_arc);
                                }
                                {
                                    let mut map_addr = clients_addresses_ref.write().await;
                                    map_addr.insert(uuid, src);
                                }

                            }

                            let datagram = &RQ_Connect_ACK_OK::new(uuid, config_ref.heart_beat_period).as_bytes();
                            result = socket_ref.send_to(datagram, src).await;

                            let sender = {
                                let map = clients_ref.read().await;
                                map.get(&uuid).unwrap().clone()
                            };

                            // New client connected spawn a task to start his personal managers
                            if !is_connected {
                                let cmd = StartManagers {
                                    clients: clients_ref,
                                    topics_subscribers: topics_subscribers_ref,
                                    clients_addresses: clients_addresses_ref,
                                    clients_structs: clients_structs_ref,
                                    b_running: b_running_ref,
                                    server_sender: socket_ref.clone(),
                                };


                                let cmd_sender = sender.clone();
                                tokio::spawn(async move {
                                    cmd_sender.send(cmd).await.expect("Failed to send StartManagers command to client manager");
                                });

                                // init his ping with this request:
                                socket_ref.send_to(&RQ_Ping::new(get_new_ping_reference(pings_ref, config_ref.clone()).await).as_bytes(), src).await.expect("Failed to send ping to client");
                            }

                            let sslrs_sender = sender.clone();
                            tokio::spawn(async move {
                                save_server_last_request_sent(sslrs_sender, uuid).await;
                            });
                            match result {
                                Ok(bytes) => {
                                    log(Info, DatagramsHandler, format!("Send {} bytes (RQ_Connect_ACK_OK) to {}", bytes, src.ip()), config_ref.clone());
                                }
                                Err(error) => {
                                    log(Error, DatagramsHandler, format!("Failed to send Connect ACK to {}.\nError: {}", src.ip(), error), config_ref.clone());
                                }
                            }
                        });

                    }// end connect
                    MessageType::DATA => {
                        // 4.2 - A user is trying to sent data to the server
                        log(Info, DatagramsHandler, format!("{} is trying to sent data", client_id), config.clone());

                        // Clone needed variable
                        let receiver_ref = receiver.clone();
                        let clients_addresses_ref = clients_addresses.clone();
                        let config_ref = config.clone();
                        let topics_subscribers_ref = topics_subscribers.clone();
                        let clients_ref = clients.clone();
                        tokio::spawn(async move {
                            handle_data(
                                receiver_ref,
                                buf,
                                client_id,
                                clients_ref,
                                clients_addresses_ref,
                                topics_subscribers_ref,
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

                        let client_sender = {
                            let map = clients.read().await;
                            map.get(&client_id).unwrap().clone()
                        };
                        let cmd = HandleDisconnect {
                            topics_subscribers: topics_subscribers.clone(),
                            clients_ref: clients.clone(),
                            clients_addresses: clients_addresses.clone(),
                            clients_structs: clients_structs.clone(),
                            client_sender,
                        };

                        let client_sender = {
                            let map = clients.read().await;
                            map.get(&client_id).unwrap().clone()
                        };
                        tokio::spawn(async move {
                            client_sender.send(cmd).await.expect("Failed to send HandleDisconnect command to client manager");
                        });
                    }
                    MessageType::HEARTBEAT => {
                        // 4.5 - A user is trying to sent an heartbeat
                        log(Info, DatagramsHandler, format!("{} is trying to sent an heartbeat", client_id), config.clone());
                        // check if client is still connected
                        /*if clients.read().await.contains_key(&client_id) {
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

                        let client_sender = {
                            let map = clients.read().await;
                            map.get(&client_id).unwrap().clone()
                        };
                        let cmd = HandleTopicRequest {
                            server_socket: receiver.clone(),
                            buffer: buf,
                            topics_subscribers: topics_subscribers.clone(),
                            root_ref: root.clone(),
                            client_sender: client_sender.clone(),
                        };

                        tokio::spawn(async move {
                            client_sender.send(cmd).await.expect("Failed to send HandleTopicRequest command to client manager");
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

    let mut interested_clients = {
        let read_client_topics = clients_topics.read().await;
        if !read_client_topics.contains_key(&data_rq.topic_id) {
            log(Warning, DataHandler, format!("Topic {} doesn't exist", data_rq.topic_id), config.clone());
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
        let data = RQ_Data::new(, data_rq.topic_id, data_rq.data.clone());
        let data = data.as_bytes();
        let client_addr = {
            // use closure to reduce the lock lifetime
            *clients_addresses.read().await.get(&client).unwrap()
        };
        let result = sender.send_to(&data, client_addr).await.unwrap();
        tokio::spawn(async move {
            save_server_last_request_sent(client_sender, client_id).await;
        });
        log(Info, DataHandler, format!("Sent {} bytes to {}", result, client_addr), config.clone());
    }
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
async fn ping_sender(
    sender: Arc<UdpSocket>,
    clients_address: Arc<RwLock<HashMap<u64, SocketAddr>>>,
    clients_senders: Arc<RwLock<HashMap<u64, Sender<ClientActions>>>>,
    pings: Arc<Mutex<HashMap<u8, u128>>>,
    b_running: Arc<bool>,
    config: Arc<Config>,
) {
    log(Info, PingSender, format!("Ping sender spawned"), config.clone());
    while *b_running {
        // get all connected ids:
        let ids: Vec<u64> = {
            clients_address.read().await.keys().map(|s| *s).collect()
        };

        for client_id in ids {
            let client_sender = {
                let map = clients_senders.read().await;
                map.get(&client_id).unwrap().clone()
            };
            let client_addr = {
                let map = clients_address.read().await;
                map.get(&client_id).unwrap().clone()
            };
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
        } // end for
        // 3 - wait 10 sec before ping everyone again
        sleep(Duration::from_secs(config.ping_period as u64)).await;
    }// end while
    // 4 - End the task
    log(Info, PingSender, format!("Ping sender destroyed"), config.clone());
}