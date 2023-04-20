// This document contain the main task of the broker. The task datagram_handler
// must never be blocked by any method ! The whole project use tokio and work
// asynchronously to allow a maximum bandwidth computing. The goal of the broker is
// to handle 100Gb/s.
// For each features the memory management and the cpu usage should be in the middle of the reflexion.
//
// @version 1.0.0 can handle 5Gb/s with 5Mo RAM
// @author : GuicLuca (lucasguichard127@gmail.com)
// date : 22/03/2023

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
use crate::client_lib::ClientActions::{HandleData, HandleDisconnect, HandleObjectRequest, HandlePong, HandleTopicRequest, StartManagers, UpdateClientLastRequest};
use crate::config::Config;
use crate::config::LogLevel::*;
use crate::datagram::*;
use crate::server_lib::*;
use crate::server_lib::LogSource::*;
use crate::types::{ClientId, ClientSender, ClientsHashMap, ObjectHashMap, PingsHashMap, ServerSocket, TopicsHashMap, TopicId};

mod config;
mod datagram;
mod server_lib;
mod client;
mod client_lib;
mod types;

use lazy_static::lazy_static;
use crate::datagram::EndConnexionReason::{SYNC_ERROR};



lazy_static! {
    static ref ISRUNNING: Arc<RwLock<bool>> = Arc::from(RwLock::from(false)); // Flag showing if the server is running or not
    static ref CONFIG: Config = Config::new(); // Unique reference to the config object

    // List of client's :
    static ref CLIENTS_SENDERS_REF: ClientsHashMap<ClientSender> = Arc::new(RwLock::new(HashMap::default())); // <Client ID, Sender> -> the sender is used to sent command through the mpsc channels
    static ref CLIENTS_STRUCTS_REF: ClientsHashMap<Arc<Mutex<Client>>> = Arc::new(RwLock::new(HashMap::default())); // <Client ID, Struct> -> used only to keep struct alive
    static ref CLIENTS_ADDRESSES_REF: ClientsHashMap<SocketAddr> = Arc::new(RwLock::new(HashMap::default())); // <Client ID, address> -> Used to send data

    // List of time reference for ping requests
    static ref PINGS_REF: PingsHashMap = Arc::new(Mutex::new(HashMap::default())); // <Ping ID, reference time in ms>

    // List of topic subscribers
    static ref TOPICS_SUBSCRIBERS_REF: TopicsHashMap<HashSet<ClientId>> = Arc::new(RwLock::new(HashMap::default())); // <Topic ID, [Clients ID]>

    // List of Objects (group of topics)
    static ref OBJECTS_TOPICS_REF: ObjectHashMap<HashSet<TopicId>> = Arc::new(RwLock::new(HashMap::default())); // <ObjectId, [TopicId]>
    static ref OBJECT_SUBSCRIBERS_REF: ObjectHashMap<HashSet<ClientId>>  = Arc::new(RwLock::new(HashMap::default())); // <ObjectId, [ClientId]>
}

#[tokio::main]
async fn main() {
    println!("[Server] Hi there !");
    log(Info, Other, format!("Config generation complete : \n{:#?}", *CONFIG));
    match local_ip() {
        Ok(ip) => println!("Server local IP and port: [{}:{}]", ip, CONFIG.port),
        Err(err) => {
            println!("Failed to get local IP:\n{}", err);
            println!("\n\nServer stopping!");
            return; // can't start the server if the local ip can't be reach
        }
    };

    /*===============================
         Init server variables
     ==============================*/

    // The socket used by the server to exchange datagrams with clients
    let socket_ref: ServerSocket = match UdpSocket::bind(format!("{}:{}", "0.0.0.0", &CONFIG.port)).await{
        Ok(socket) => {
             Arc::new(socket)
        }
        Err(err) => {
            log(Error, Other, format!("Failed to bind the socket to the address {}:{}. Error:\n{}", "0.0.0.0", &CONFIG.port, err));
            return;
        }
    };


    // =============================
    //    Spawning async functions
    // =============================
    update_server_status(true).await;

    let (ping_sender, datagram_handler) = { // Closure used to reduce clone lifetime
        // clone needed value for the second task
        let datagram_socket = socket_ref.clone();

        let ping_sender = tokio::spawn(async move {
            ping_sender(
                socket_ref,
            ).await;
        });
        // ... and then spawn the second task
        let datagram_handler = tokio::spawn(async move {
            datagrams_handler(
                datagram_socket,
            ).await;
        });
        (ping_sender, datagram_handler)
    };

    log(Info, Other, format!("Server is running ..."));


    join!(datagram_handler, ping_sender).0.expect("Error while joining the tasks");

    update_server_status(false).await;

    log(Info, Other, format!("Server has stopped ... Running status : {}", ISRUNNING.read().await));
}


/**
 * This method handle every incoming datagram in the broker and start/stop task
 * to compute needed data.
 *
 * @param receiver: ServerSocket, The server socket use to exchange data
 */
async fn datagrams_handler(
    receiver: ServerSocket,
) {
    log(Info, DatagramsHandler, format!("Datagrams Handler spawned"));
    // loop receiving listening for datagrams

    // Init the server status
    let mut server_is_running = {
        *ISRUNNING.read().await
    };
    while server_is_running {
        // 1 - create an empty buffer of size 1024
        let mut buf = [0; 1024];

        // 2 - Wait for bytes reception
        match receiver.recv_from(&mut buf).await {
            // 3 - Once bytes are received, check for errors
            Ok((n, src)) => {
                // 4 - if OK match on the first byte (MESSAGE_TYPE)
                log(Info, DatagramsHandler, format!("Received {} bytes from {}", n, src));

                // 4 - Get the client id source of the request
                let client_id: ClientId = match get_client_id(src).await {
                    //  5 - Do not handle the datagram if the source is not auth and is not trying to connect
                    None => {
                        if MessageType::from(buf[0]) != MessageType::CONNECT
                        {
                            log(Warning, DatagramsHandler, format!("Datagrams dropped from {}. Error : Not connected and trying to {}.", src.ip(), display_message_type(MessageType::from(buf[0]))));
                            continue;
                        }
                        // set client_id to 0 for a default value
                        0
                    }
                    Some(id) => {
                        // update his last time he sent a request
                        let client_sender = {
                            let map = CLIENTS_SENDERS_REF.read().await;
                            match map.get(&id){
                                None => {
                                    // Client id is not set in the senders hashsets :
                                    // SYNC ERROR clients_hashset are not sync so try to clean the id from every client sets
                                    log(Error, DatagramsHandler, format!("The client {} has common hashset sync issue. Closing the connexion with an connect_NACK", src.ip()));
                                    tokio::spawn(async move {
                                       try_remove_client_from_set(id).await;
                                    });
                                    match receiver.send_to(
                                        &RQ_Shutdown::new(SYNC_ERROR).as_bytes(),
                                        src
                                    ).await {
                                        Err(err) => {
                                            log(Error, DatagramsHandler, format!("Failed to send connexion error to {}. The client has common hashset sync issue. Error:\n{}", src.ip(), err));
                                        }
                                        _ => {}
                                    }
                                    return;
                                }
                                Some(sender) => {
                                    // return the clone of the client sender
                                    sender.clone()
                                }
                            }
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
                        // set client_id with the found one
                        id
                    }
                };

                // 6 - Match the datagram type and process it
                match MessageType::from(buf[0]) {
                    MessageType::CONNECT => {
                        // 4.1 - A user is trying to connect to the server
                        log(Info, DatagramsHandler, format!("{} is trying to connect", src.ip()));

                        let socket_ref = receiver.clone();
                        tokio::spawn(async move {
                            let (is_connected, current_id) = already_connected(&src.ip()).await;
                            let uuid;
                            let result;

                            if is_connected {
                                // 2 - User already connected, return the same data as previous connection
                                uuid = current_id;
                                log(Info, DatagramsHandler, format!("{} was already a client, UUID : {}", src.ip(), uuid));
                            } else {
                                //3 - it's a new connection : generate a new id
                                uuid = get_new_id();
                                log(Info, DatagramsHandler, format!("{} is now a client, UUID : {}", src.ip(), uuid));

                                // 3.1 - Create a chanel to exchange commands through
                                let (sender, receiver) = mpsc::channel::<ClientActions>(32);
                                let new_client = Client::new(uuid, src, receiver);
                                let new_client_arc: Arc<Mutex<Client>> = Arc::new(Mutex::new(new_client)); // clone new_client and wrap in Arc and Mutex

                                // 3.2 - fill server arrays to keep in memory the connection
                                {
                                    let mut map = CLIENTS_SENDERS_REF.write().await;
                                    map.insert(uuid, sender);
                                }
                                {
                                    let manager_ref = new_client_arc.clone();
                                    tokio::spawn(async move {
                                        manager_ref.lock().await.manager().await;
                                    });
                                    let mut map = CLIENTS_STRUCTS_REF.write().await;
                                    map.insert(uuid, new_client_arc);
                                }
                                {
                                    let mut map_addr = CLIENTS_ADDRESSES_REF.write().await;
                                    map_addr.insert(uuid, src);
                                }
                            }
                            let sender = {
                                let map = CLIENTS_SENDERS_REF.read().await;
                                map.get(&uuid).unwrap().clone()
                            };
                            let datagram = &RQ_Connect_ACK_OK::new(uuid, CONFIG.heart_beat_period).as_bytes();
                            result = send_datagram(socket_ref.clone(), datagram, src, sender.clone()).await;

                            // New client connected spawn a task to start his personal managers
                            if !is_connected {
                                let cmd = StartManagers {
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
                                    &RQ_Ping::new(get_new_ping_reference().await).as_bytes(),
                                    src,
                                    sender.clone(),
                                ).await.unwrap();
                            } // End if
                            match result {
                                Ok(bytes) => {
                                    log(Info, DatagramsHandler, format!("Send {} bytes (RQ_Connect_ACK_OK) to {}", bytes, src.ip()));
                                }
                                Err(error) => {
                                    log(Error, DatagramsHandler, format!("Failed to send Connect ACK to {}.\nError: {}", src.ip(), error));
                                }
                            }
                        });
                    }// end connect
                    MessageType::DATA => {
                        // 4.2 - A user is trying to sent data to the server
                        log(Info, DatagramsHandler, format!("{} is trying to sent data", client_id));

                        let cmd = HandleData {
                            sender: receiver.clone(),
                            buffer: buf,
                        };
                        let client_sender = {
                            let map = CLIENTS_SENDERS_REF.read().await;
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
                        log(Info, DatagramsHandler, format!("{} is trying to open a new stream", client_id));
                    }
                    MessageType::SHUTDOWN => {
                        // 4.4 - A user is trying to shutdown the connexion with the server
                        log(Info, DatagramsHandler, format!("{} is trying to shutdown the connexion with the server", client_id));

                        let cmd = HandleDisconnect {};

                        let client_sender = {
                            let map = CLIENTS_SENDERS_REF.read().await;
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
                        log(Info, DatagramsHandler, format!("{} is trying to sent an heartbeat", client_id));
                    }
                    MessageType::OBJECT_REQUEST => {
                        // 4.6 - A user is trying to request an object
                        log(Info, DatagramsHandler, format!("{} is trying to request an object", client_id));
                        let client_sender = {
                            let map = CLIENTS_SENDERS_REF.read().await;
                            map.get(&client_id).unwrap().clone()
                        };
                        let cmd = HandleObjectRequest {
                            server_socket: receiver.clone(),
                            buffer: buf,
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
                    MessageType::TOPIC_REQUEST => {
                        // 4.7 - A user is trying to request a new topic
                        log(Info, DatagramsHandler, format!("{} is trying to request a new topic", client_id));

                        let client_sender = {
                            let map = CLIENTS_SENDERS_REF.read().await;
                            map.get(&client_id).unwrap().clone()
                        };
                        let cmd = HandleTopicRequest {
                            server_socket: receiver.clone(),
                            buffer: buf,
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
                        log(Info, DatagramsHandler, format!("{} is trying to answer a ping request", client_id));
                        let time = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis(); // Current time in ms

                        // If the client is offline, don't compute the request
                        if !is_online(client_id).await { continue; };

                        let cmd = HandlePong {
                            ping_id: buf[1],
                            current_time: time,
                        };
                        // send the command :
                        let clients_ref = CLIENTS_SENDERS_REF.clone();
                        tokio::spawn(async move {
                            match clients_ref.read().await.get(&client_id).unwrap().send(cmd).await {
                                Ok(_) => {}
                                Err(_) => {
                                    return;
                                }
                            };
                        });
                    }
                    MessageType::SERVER_STATUS => {
                        // get common information to answer the request
                        let connected_clients = {
                            CLIENTS_SENDERS_REF.read().await.len()
                        };

                        // check if the request is from a connected client or not :
                        if client_id == 0 {
                            log(Info, DatagramsHandler, format!("Received SERVER_STATUS packet from {}", src.ip()));
                            // not connected : send the datagram manually to the source
                            receiver.send_to(
                                &RQ_ServerStatus_ACK::new(server_is_running, connected_clients as u64).as_bytes(),
                                src
                            ).await.unwrap();
                        }else{
                            log(Info, DatagramsHandler, format!("Received SERVER_STATUS packet from {}", client_id));
                            // Connected : send the response with the method to update client life signe timestamp
                            let client_sender = {
                                CLIENTS_SENDERS_REF.read().await.get(&client_id).unwrap().clone()
                            };
                            send_datagram(
                                receiver.clone(),
                                &RQ_ServerStatus_ACK::new(server_is_running, connected_clients as u64).as_bytes(),
                                src,
                                client_sender,
                            ).await.unwrap();
                        }
                    }
                    MessageType::SERVER_STATUS_ACK | MessageType::TOPIC_REQUEST_ACK | MessageType::OBJECT_REQUEST_ACK | MessageType::CONNECT_ACK | MessageType::HEARTBEAT_REQUEST | MessageType::PING | MessageType::TOPIC_REQUEST_NACK | MessageType::OBJECT_REQUEST_NACK => {
                        // 4.9 - invalid datagrams for the server
                        log(Warning, DatagramsHandler, format!("{} has sent an invalid datagram.", client_id));
                    }
                    MessageType::UNKNOWN => {
                        log(Warning, DatagramsHandler, format!("Received unknown packet from {}", src.ip()));
                    }
                }
            }
            Err(e) => {
                log(Error, DatagramsHandler, format!("Error: {}", e));
            }
        }

        // update the server status before looping again
        server_is_running = {
            *ISRUNNING.read().await
        };
    } // end while
}

/**
This method send ping request to every connected clients
@param sender ServerSocket : An atomic reference of the UDP socket of the server

@return None
 */
async fn ping_sender(
    sender: ServerSocket,
) {
    log(Info, PingSender, format!("Ping sender spawned"));

    // Init the server status
    let mut server_is_running = {
        *ISRUNNING.read().await
    };

    while server_is_running {
        // get all connected ids:
        let ids: Vec<ClientId> = {
            CLIENTS_ADDRESSES_REF.read().await.keys().map(|s| *s).collect()
        };

        for client_id in ids {
            let client_sender = {
                let map = CLIENTS_SENDERS_REF.read().await;
                map.get(&client_id).unwrap().clone()
            };
            let client_addr = {
                let map = CLIENTS_ADDRESSES_REF.read().await;
                map.get(&client_id).unwrap().clone()
            };

            // 2 - Send a ping request to the client
            match send_datagram(
                sender.clone(),
                &RQ_Ping::new(get_new_ping_reference().await).as_bytes(),
                client_addr,
                client_sender.clone(),
            ).await {
                Ok(bytes) => {
                    log(Info, PingSender, format!("Send {} bytes to {}", bytes, client_id));
                }
                Err(error) => {
                    log(Error, PingSender, format!("Failed to send ping request to {}.\n{}", client_id, error));
                }
            }
        } // end for
        // 3 - wait 10 sec before ping everyone again
        sleep(Duration::from_secs(CONFIG.ping_period as u64)).await;

        // 4 - Update the server status before running again.
        server_is_running = {
            *ISRUNNING.read().await
        };
    }// end while
    // 5 - End the task
    log(Info, PingSender, format!("Ping sender destroyed"));
}