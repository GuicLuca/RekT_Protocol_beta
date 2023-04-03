use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use local_ip_address::local_ip;
use tokio::{join, net::UdpSocket};
use tokio::sync::{mpsc, Mutex};
use tokio::sync::RwLock;
use tokio::time::sleep;

use crate::client::Client;
use crate::client_lib::{ClientActions, now_ms};
use crate::client_lib::ClientActions::{HandleData, HandleDisconnect, HandlePong, HandleTopicRequest, StartManagers, UpdateClientLastRequest};
use crate::config::Config;
use crate::config::LogLevel::*;
use crate::datagram::*;
use crate::server_lib::*;
use crate::server_lib::LogSource::*;
use crate::types::{ClientId, ClientSender, ClientsHashMap, PingsHashMap, ServerSocket, TopicsHashMap};

mod config;
mod datagram;
mod server_lib;
mod client;
mod client_lib;
mod types;

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
    let socket_ref: ServerSocket = Arc::new(socket);

    // List of client's :
    let clients_ref: ClientsHashMap<ClientSender> = Arc::new(RwLock::new(HashMap::default())); // <Client ID, Sender> -> the sender is used to sent command through the mpsc channels
    let clients_structs: ClientsHashMap<Arc<Mutex<Client>>> = Arc::new(RwLock::new(HashMap::default())); // <Client ID, Struct> -> used only to keep struct alive
    let clients_address_ref: ClientsHashMap<SocketAddr> = Arc::new(RwLock::new(HashMap::default())); // <Client ID, address> -> Used to send data

    // List of time reference for ping requests
    let pings_ref: PingsHashMap = Arc::new(Mutex::new(HashMap::default())); // <Ping ID, reference time in ms>

    // List of topic subscribers
    let topics_subscribers_ref: TopicsHashMap<HashSet<ClientId>> = Arc::new(RwLock::new(HashMap::default())); // <Topic ID, [Clients ID]>

    // Root topic which can be subscribed by clients
    // Every topics have sub topic to them, you can go through each one like in a tree
    //let objects_topic: TopicsHashMap<HashSet<ClientId>> = Arc::new(RwLock::new(HashMap::new()));


    log(Info, Other, format!("Server variables successfully initialized"), config.clone());

    // =============================
    //    Spawning async functions
    // =============================
    b_running = Arc::from(true);
    let (ping_sender, datagram_handler) = { // Closure used to reduce clone lifetime
        let config_ref = config.clone();
        // clone needed value for the second task
        let datagram_socket = socket_ref.clone();
        let datagram_clients = clients_ref.clone();
        let datagram_clients_address = clients_address_ref.clone();
        let datagram_pings = pings_ref.clone();
        let datagram_b_running = b_running.clone();
        let datagram_config = config_ref.clone();

        let ping_sender = tokio::spawn(async move {
            ping_sender(
                socket_ref,
                clients_address_ref,
                clients_ref,
                pings_ref,
                b_running,
                config_ref,
            ).await;
        });
        // ... and then spawn the second task
        let datagram_handler = tokio::spawn(async move {
            datagrams_handler(
                datagram_socket,
                datagram_clients,
                datagram_clients_address,
                datagram_pings,
                datagram_b_running,
                topics_subscribers_ref,
                clients_structs,
                datagram_config,
            ).await;
        });
        (ping_sender, datagram_handler)
    };

    log(Info, Other, format!("Server is running ..."), config.clone());


    join!(datagram_handler, ping_sender).0.expect("Error while joining the tasks");

    b_running = Arc::from(false);
    log(Info, Other, format!("Server has stopped ... Running status : {}", b_running), config.clone());
}


/**
 * This method handle every incoming datagram in the broker and start/stop task
 * to compute needed data.
 *
 * @param receiver: ServerSocket, The server socket use to exchange data
 * @param clients: ClientsHashMap<ClientSender>, HashMap containing every client channel.
 * @param clients_addresses: ClientsHashMap<SocketAddr>, HashMap containing every client address.
 * @param pings: PingsHashMap, hashMap containing every ping references
 * @param b_running: Arc<bool>, State of the server
 * @param topics_subscriber: TopicsHashMap<HashSet<ClientId>>, HashMap containing every client Id that are subscribed to a topic.
 * @param clients_structs: ClientsHashMap<Arc<Mutex<Client>>>, HashMap containing every client struct.
 * @param config: Arc<Config>, The config reference shared to every task
 */
async fn datagrams_handler(
    receiver: ServerSocket,
    clients: ClientsHashMap<ClientSender>,
    clients_addresses: ClientsHashMap<SocketAddr>,
    pings: PingsHashMap,
    b_running: Arc<bool>,
    topics_subscribers: TopicsHashMap<HashSet<ClientId>>,
    clients_structs: ClientsHashMap<Arc<Mutex<Client>>>,
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

                // 4 - Get the client id source of the request
                let client_id_found = get_client_id(src, clients_addresses.clone().read().await.to_owned()).await;
                let mut client_id = 0;
                //  5 - Do not handle the datagram if the source is not auth and is not trying to connect
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
                        match client_sender.send(cmd).await {
                            Ok(_) => {}
                            Err(_) => {
                                return;
                            }
                        };
                    });
                }

                // 6 - Match the datagram type and process it
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
                        tokio::spawn(async move {
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
                                let new_client_arc: Arc<Mutex<Client>> = Arc::new(Mutex::new(new_client)); // clone new_client and wrap in Arc and Mutex

                                // 3.2 - fill server arrays to keep in memory the connection
                                {
                                    let mut map = clients_ref.write().await;
                                    map.insert(uuid, sender);
                                }
                                {
                                    let manager_ref = new_client_arc.clone();
                                    let config_ref_task = config_ref.clone();
                                    tokio::spawn(async move {
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
                            let sender = {
                                let map = clients_ref.read().await;
                                map.get(&uuid).unwrap().clone()
                            };
                            let datagram = &RQ_Connect_ACK_OK::new(uuid, config_ref.heart_beat_period).as_bytes();
                            result = send_datagram(socket_ref.clone(), datagram, src, sender.clone()).await;

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
                                    match cmd_sender.send(cmd).await {
                                        Ok(_) => {}
                                        Err(_) => {
                                            return;
                                        }
                                    };
                                });

                                // init his ping with this request:
                                send_datagram(
                                    socket_ref,
                                    &RQ_Ping::new(get_new_ping_reference(pings_ref, config_ref.clone()).await).as_bytes(),
                                    src,
                                    sender.clone(),
                                ).await.unwrap();
                            } // End if
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

                        let cmd = HandleData {
                            sender: receiver.clone(),
                            buffer: buf,
                            clients: clients.clone(),
                            clients_addresses: clients_addresses.clone(),
                            config: config.clone(),
                            topics_subscribers: topics_subscribers.clone(),
                        };
                        let client_sender = {
                            let map = clients.read().await;
                            map.get(&client_id).unwrap().clone()
                        };
                        match client_sender.send(cmd).await {
                            Ok(_) => {}
                            Err(_) => {
                                return;
                            }
                        };
                    }
                    MessageType::OPEN_STREAM => {
                        // 4.3 - A user is trying to open a new stream
                        log(Info, DatagramsHandler, format!("{} is trying to open a new stream", client_id), config.clone());
                    }
                    MessageType::SHUTDOWN => {
                        // 4.4 - A user is trying to shutdown the connexion with the server
                        log(Info, DatagramsHandler, format!("{} is trying to shutdown the connexion with the server", client_id), config.clone());

                        let cmd = HandleDisconnect {
                            topics_subscribers: topics_subscribers.clone(),
                            clients_ref: clients.clone(),
                            clients_addresses: clients_addresses.clone(),
                            clients_structs: clients_structs.clone(),
                        };

                        let client_sender = {
                            let map = clients.read().await;
                            map.get(&client_id).unwrap().clone()
                        };
                        tokio::spawn(async move {
                            match client_sender.send(cmd).await {
                                Ok(_) => {}
                                Err(_) => {
                                    return;
                                }
                            };
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
                            client_sender: client_sender.clone(),
                        };

                        tokio::spawn(async move {
                            match client_sender.send(cmd).await {
                                Ok(_) => {}
                                Err(_) => {
                                    return;
                                }
                            };
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
                            match clients_ref.read().await.get(&client_id).unwrap().send(cmd).await {
                                Ok(_) => {}
                                Err(_) => {
                                    return;
                                }
                            };
                        });
                    }
                    MessageType::TOPIC_REQUEST_ACK | MessageType::OBJECT_REQUEST_ACK | MessageType::CONNECT_ACK | MessageType::HEARTBEAT_REQUEST | MessageType::PING | MessageType::TOPIC_REQUEST_NACK => {
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

/** TODO : doc
This method send ping request to every connected clients
@param sender ServerSocket : An atomic reference of the UDP socket of the server
@param client_id u64 : The client identifier.
@param clients: ClientsHashMap<SocketAddr> : An atomic reference of the pings HashMap. The map is protected by a rwlock to be thread safe
@param pings PingsHashMap : An atomic reference of the pings HashMap. The map is protected by a mutex to be thread safe
@param b_running Arc<bool> : An atomic reference of the server status to stop the "thread" if server is stopping
@param clients_ping_ref Arc<Mutex<HashMap<u64, u128>>> : An atomic reference of client's ping hashmap, protected by a mutex to be thread safe

@return None
 */
async fn ping_sender(
    sender: ServerSocket,
    clients_address: ClientsHashMap<SocketAddr>,
    clients_senders: ClientsHashMap<ClientSender>,
    pings: PingsHashMap,
    b_running: Arc<bool>,
    config: Arc<Config>,
) {
    log(Info, PingSender, format!("Ping sender spawned"), config.clone());
    while *b_running {
        // get all connected ids:
        let ids: Vec<ClientId> = {
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
            match send_datagram(
                sender.clone(),
                &RQ_Ping::new(get_new_ping_reference(pings.clone(), config.clone()).await).as_bytes(),
                client_addr,
                client_sender.clone(),
            ).await {
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