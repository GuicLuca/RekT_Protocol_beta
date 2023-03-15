use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::string::String;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use local_ip_address::local_ip;
use tokio::{join, net::UdpSocket};
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::time::sleep;
use crate::ps_config::Config;
use crate::ps_config::LogLevel::*;
use crate::ps_datagram_structs::*;
use crate::ps_server_lib::*;
use crate::ps_server_lib::LogSource::*;
use crate::topic_v2::TopicV2;

mod ps_datagram_structs;
mod topic_v2;
mod ps_server_lib;
mod ps_config;

#[tokio::main]
async fn main() {
    println!("[Server] Hi there !");
    let config: Arc<Config> = Arc::new(Config::new());
    //let port = ps_common::get_cli_input("[Server] Hi there ! Chose the port of for the server :", "Cannot get the port form the cli input.", None, None, true);
    log(Info, Other, format!("Config generation complete : \n{:#?}", config), config.clone());

    log(Info, Other, format!("The ip of the server is {}:{}", local_ip().unwrap(), config.port), config.clone());

    /* ===============================
           Init all server variable
       ==============================*/
    // Flag showing if the server is running or not
    #[allow(unused)]
    let mut b_running;

    // The socket used by the server to exchange datagrams with clients
    let socket = UdpSocket::bind(format!("{}:{}", "0.0.0.0", config.port.parse::<i16>().unwrap())).await.unwrap();
    let socket_ref = Arc::new(socket);

    // Address and port of the server
    //let address = String::new();
    //let port: i16 = port.parse::<i16>().unwrap();

    // Root topic which can be subscribed by clients
    // Every topics have sub topic to them, you can go through each one like in a tree
    let root = RwLock::new(TopicV2::new(1, "/".to_string())); // default root topics is "/"
    let root_ref = Arc::new(root);


    // List of clients represented by their address and their ID
    let clients: RwLock<HashMap<u64, SocketAddr>> = RwLock::new(HashMap::default()); // <Client ID, address>
    let clients_ref = Arc::new(clients);


    // List of clients ping
    let clients_ping_ref: Arc<RwLock<HashMap<u64, u128>>> = Arc::new(RwLock::new(HashMap::default())); // <Client ID, Ping in ms>
    let pings_ref: Arc<Mutex<HashMap<u8, u128>>> = Arc::new(Mutex::new(HashMap::default())); // <Ping ID, reference time in ms>

    // List of client heartbeat_status
    let client_has_heartbeat_ref: Arc<RwLock<HashMap<u64, bool>>> = Arc::new(RwLock::new(HashMap::default())); // <Client ID, has_heartbeat>

    // List of client's subscribed topic
    let clients_topics_ref: Arc<RwLock<HashMap<u64, HashSet<u64>>>> = Arc::new(RwLock::new(HashMap::default())); // <Client ID, [TopicsID]>

    log(Info, Other, format!("Server variables successfully initialized"), config.clone());

    // =============================
    //    Spawning async functions
    // =============================
    b_running = Arc::from(true);
    let datagram_handler = tokio::spawn(datagrams_handler(
        socket_ref.clone(),
        clients_ref.clone(),
        root_ref.clone(),
        pings_ref.clone(),
        clients_ping_ref.clone(),
        b_running.clone(),
        client_has_heartbeat_ref.clone(),
        clients_topics_ref.clone(),
        config.clone(),
    ));
    //let ping_sender = tokio::spawn(ping_sender(socket_ref.clone(), clients_ref.clone(), pings_ref.clone(),b_running.clone()));

    log(Info, Other, format!("Server is running ..."), config.clone());

    #[allow(unused)]
        let res1 = join!(datagram_handler);

    b_running = Arc::from(false);
    log(Info, Other, format!("Server has stopped ... Running status : {}", b_running), config.clone());
}

/**
This method handle every incoming datagram in the broker
@param receiver Arc<UdpSocket> : An atomic reference of the UDP socket of the server
@param clients_ref Arc<RwLock<HashMap<u64, SocketAddr>>> : An atomic reference of the clients HashMap. The map is protected by a rwLock to be thread safe
@param root_ref Arc<RwLock<TopicV2>> : An atomic reference of root topics, protected by a rwlock to be thread safe
@param pings_ref Arc<Mutex<HashMap<u64, u128>>> : An atomic reference of the pings time references, protected by a mutex to be thread safe
@param clients_ping_ref Arc<Mutex<HashMap<u64, u128>>> : An atomic reference of client's ping hashmap, protected by a rwLock to be thread safe
@param b_running Arc<bool> : An atomic reference of the server status to stop the "thread" if server is stopping
@param client_has_heartbeat_ref Arc<RwLock<HashMap<u64, bool>>> : An atomic reference of the clients_heartbeat status.

@return none
 */
async fn datagrams_handler(
    receiver: Arc<UdpSocket>,
    clients_ref: Arc<RwLock<HashMap<u64, SocketAddr>>>,
    root_ref: Arc<RwLock<TopicV2>>,
    pings_ref: Arc<Mutex<HashMap<u8, u128>>>,
    clients_ping_ref: Arc<RwLock<HashMap<u64, u128>>>,
    b_running: Arc<bool>,
    client_has_heartbeat_ref: Arc<RwLock<HashMap<u64, bool>>>,
    clients_topics_ref: Arc<RwLock<HashMap<u64, HashSet<u64>>>>,
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

                if (MessageType::from(buf[0]) != MessageType::CONNECT) &&
                    get_client_id(&src, clients_ref.clone()).await.is_none()
                {
                    // Do not handle the datagram if the source is not auth and is not trying to connect
                    log(Warning, DatagramsHandler, format!("Datagrams dropped from {}. Error : Not connected and trying to {}.", src.ip(), display_message_type(MessageType::from(buf[0]))), config.clone());
                    continue;
                }

                match MessageType::from(buf[0]) {
                    MessageType::CONNECT => {
                        // 4.1 - A user is trying to connect to the server
                        log(Info, DatagramsHandler, format!("{} is trying to connect", src.ip()), config.clone());
                        let already_client = handle_connect(src, clients_ref.clone(), receiver.clone(), config.clone()).await;
                        let id = get_client_id(&src, clients_ref.clone()).await.unwrap();
                        if !already_client {
                            #[allow(unused)]
                                let sender = tokio::spawn(ping_sender(receiver.clone(), id, clients_ref.clone(), pings_ref.clone(), clients_ping_ref.clone(), b_running.clone(), config.clone()));
                            #[allow(unused)]
                                let heartbeat = tokio::spawn(heartbeat_checker(receiver.clone(), id, clients_ref.clone(), client_has_heartbeat_ref.clone(), b_running.clone(), clients_topics_ref.clone(), config.clone()));
                        }
                    }
                    MessageType::DATA => {
                        // 4.2 - A user is trying to sent data to the server
                        let client_id = get_client_id(&src, clients_ref.clone()).await.unwrap();
                        log(Info, DatagramsHandler, format!("{} is trying to sent data", client_id), config.clone());
                        #[allow(unused)]
                            let handler = tokio::spawn(handle_data(receiver.clone(), buf, client_id, clients_ref.clone(), clients_topics_ref.clone(), config.clone()));
                    }
                    MessageType::OPEN_STREAM => {
                        // 4.3 - A user is trying to open a new stream
                        let client_id = get_client_id(&src, clients_ref.clone()).await.unwrap();
                        log(Info, DatagramsHandler, format!("{} is trying to open a new stream", client_id), config.clone());
                    }
                    MessageType::SHUTDOWN => {
                        // 4.4 - A user is trying to shutdown the connexion with the server
                        let client_id = get_client_id(&src, clients_ref.clone()).await.unwrap();
                        log(Info, DatagramsHandler, format!("{} is trying to shutdown the connexion with the server", client_id), config.clone());
                        handle_disconnect(clients_ref.clone(), client_id, clients_topics_ref.clone(), config.clone()).await;
                    }
                    MessageType::HEARTBEAT => {
                        // 4.5 - A user is trying to sent an heartbeat
                        let client_id = get_client_id(&src, clients_ref.clone()).await.unwrap();
                        log(Info, DatagramsHandler, format!("{} is trying to sent an heartbeat", client_id), config.clone());
                        // check if client is still connected
                        if clients_ref.read().await.contains_key(&client_id) {
                            client_has_heartbeat_ref.write().await.entry(client_id)
                                .and_modify(|v| *v = true)
                                .or_insert(true);
                        }
                    }
                    MessageType::OBJECT_REQUEST => {
                        // 4.6 - A user is trying to request an object
                        let client_id = get_client_id(&src, clients_ref.clone()).await.unwrap();
                        log(Info, DatagramsHandler, format!("{} is trying to request an object", client_id), config.clone());
                    }
                    MessageType::TOPIC_REQUEST => {
                        // 4.7 - A user is trying to request a new topic
                        let client_id = get_client_id(&src, clients_ref.clone()).await.unwrap();
                        log(Info, DatagramsHandler, format!("{} is trying to request a new topic", client_id), config.clone());
                        #[allow(unused)]
                            let handler = tokio::spawn(topics_request_handler(receiver.clone(), buf, client_id, clients_ref.clone(), clients_topics_ref.clone(), root_ref.clone(), config.clone()));
                    }
                    MessageType::PONG => {
                        // 4.8 - A user is trying to answer a ping request
                        let client_id = get_client_id(&src, clients_ref.clone()).await.unwrap();
                        log(Info, DatagramsHandler, format!("{} is trying to answer a ping request", client_id), config.clone());
                        let time = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis(); // Current time in ms
                        let client_id = get_client_id(&src, clients_ref.clone()).await.unwrap();

                        handle_pong(client_id, buf[1], time, pings_ref.clone(), clients_ping_ref.clone(), clients_ref.clone(), config.clone()).await;
                    }
                    MessageType::TOPIC_REQUEST_ACK | MessageType::OBJECT_REQUEST_ACK | MessageType::CONNECT_ACK | MessageType::HEARTBEAT_REQUEST | MessageType::PING => {
                        // 4.9 - invalid datagrams for the server
                        let client_id = get_client_id(&src, clients_ref.clone()).await.unwrap();
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
    clients: Arc<RwLock<HashMap<u64, SocketAddr>>>,
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
        let client_addr = *clients.read().await.get(&client).unwrap();
        let data = RQ_Data::new(data_rq.topic_id, data_rq.data.clone());
        let data = data.as_bytes();
        let result = sender.send_to(&data, client_addr).await.unwrap();
        log(Info, DataHandler, format!("Sent {} bytes to {}", result, client_addr), config.clone());
    }
}


/**
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
    client_id: u64,
    clients: Arc<RwLock<HashMap<u64, SocketAddr>>>,
    pings: Arc<Mutex<HashMap<u8, u128>>>,
    clients_ping_ref: Arc<RwLock<HashMap<u64, u128>>>,
    b_running: Arc<bool>,
    config: Arc<Config>,
) {
    log(Info, PingSender, format!("Ping sender spawned for {}", client_id), config.clone());
    // 1 - get the client address
    let client_addr = *clients.read().await.get(&client_id).unwrap();

    // 2 - Loop while server is running and client is online
    while *b_running && is_online(client_id, clients.clone()).await {
        // 3 - Send a ping request to the client
        let result = sender.send_to(&RQ_Ping::new(get_new_ping_reference(pings.clone(), config.clone()).await).as_bytes(), client_addr).await;
        match result {
            Ok(bytes) => {
                log(Info, PingSender, format!("Send {} bytes to {}", bytes, client_id), config.clone());
            }
            Err(error) => {
                log(Error, PingSender, format!("Failed to send ping request to {}.\n{}", client_id, error), config.clone());
            }
        }
        // 4 - wait 10 sec before ping everyone again
        sleep(Duration::from_secs(config.ping_period as u64)).await;
    }
    // 5 - End the task
    clients_ping_ref.write().await.remove(&client_id);
    log(Info, PingSender, format!("Ping sender destroyed for {}", client_id), config.clone());
}

/**
This method check if heartbeat are sent correctly and else close the client session
@param sender Arc<UdpSocket> : An atomic reference of the UDP socket of the server
@param client_id u64 : The client identifier.
@param clients Arc<RwLock<HashMap<u64, SocketAddr>>> : An atomic reference of the pings HashMap. The map is protected by a rwlock to be thread safe
@param client_has_heartbeat_ref Arc<RwLock<HashMap<u64, bool>>> : An atomic reference of the heartbeat_status HashMap. The map is protected by a rwlock to be thread safe
@param b_running Arc<bool> : An atomic reference of the server status to stop the "thread" if server is stopping

@return None
 */
async fn heartbeat_checker(
    sender: Arc<UdpSocket>,
    client_id: u64,
    clients: Arc<RwLock<HashMap<u64, SocketAddr>>>,
    client_has_heartbeat_ref: Arc<RwLock<HashMap<u64, bool>>>,
    b_running: Arc<bool>,
    client_topics: Arc<RwLock<HashMap<u64, HashSet<u64>>>>,
    config: Arc<Config>,
) {
    log(Info, HeartbeatChecker, format!("HeartbeatChecker sender spawned for {}", client_id), config.clone());
    // 1 - Init local variables
    let mut missed_heartbeats: u8 = 0; // used to count how many HB are missing
    let client_addr = *clients.read().await.get(&client_id).unwrap();

    // 2 - Loop while server is running and client is online
    while *b_running && is_online(client_id, clients.clone()).await {
        // 3 - waite for the heartbeat period
        sleep(Duration::from_secs(config.heart_beat_period as u64)).await;

        // Do the following in a special code space to lock the "has_sent_heartbeat" value the same before reseting it.
        // It prevent overriding a new incoming Heartbeat request.
        let has_sent_heartbeat = {
            let mut write_client_hshb = client_has_heartbeat_ref.write().await;
            // 4 - get the heartbeat_status
            let has_sent_heartbeat = match write_client_hshb.get(&client_id) {
                Some(bool_value) => *bool_value,
                None => false, // return false if client_id is not present in the map
            };

            // 5 - reset the value
            write_client_hshb.entry(client_id)
                .and_modify(|v| *v = false)
                .or_insert(false);

            has_sent_heartbeat
        };

        // 6 - check if client has sent heartbeat
        if !has_sent_heartbeat {
            // increase the missing packet
            missed_heartbeats += 1;
            log(Info, HeartbeatChecker, format!("{} hasn't sent heartbeat {} times", client_id, missed_heartbeats), config.clone());
            if missed_heartbeats == 2 {
                // 7 - send an heartbeat request
                let result = sender.send_to(&RQ_Heartbeat_Request::new().as_bytes(), client_addr).await;
                match result {
                    Ok(bytes) => {
                        log(Info, HeartbeatChecker, format!("Send {} bytes (RQ_Heartbeat_Request) to {}", bytes, client_id), config.clone());
                    }
                    Err(error) => {
                        log(Error, HeartbeatChecker, format!("Failed to send RQ_HeartBeat_Request to {}. \nError: {}", client_id, error), config.clone());
                    }
                }
            } else if missed_heartbeats == 4 {
                // 8 - No Heartbeat for 4 period = end the client connection.
                // 8.1 - Send shutdown request
                let result = sender.send_to(&RQ_Shutdown::new(EndConnexionReason::SHUTDOWN).as_bytes(), client_addr).await;
                match result {
                    Ok(bytes) => {
                        log(Info, HeartbeatChecker, format!("Send {} bytes (RQ_Heartbeat_Request) to {}", bytes, client_id), config.clone());
                    }
                    Err(error) => {
                        log(Error, HeartbeatChecker, format!("Failed to send RQ_Shutdown to {}. \nError: {}", client_id, error), config.clone());
                    }
                }

                // 8.2 - Remove the client from the main array
                handle_disconnect(clients.clone(), client_id, client_topics.clone(), config.clone()).await;
                // client's task will end automatically
            }
        } else {
            // 7bis - reset flags variable
            missed_heartbeats = 0;
            let result = sender.send_to(&RQ_Heartbeat::new().as_bytes(), client_addr).await;
            match result {
                Ok(_) => {
                    log(Info, HeartbeatChecker, format!("Respond to client bytes (HeartBeat) to {}", client_id), config.clone());
                }
                Err(error) => {
                    log(Error, HeartbeatChecker, format!("Failed to send RQ_HeartBeat to {}.\nError: {}", client_id, error), config.clone());
                }
            }
        }
    }

    // 9 - End the task
    client_has_heartbeat_ref.write().await.remove(&client_id);
    log(Info, HeartbeatChecker, format!("Heartbeat checker destroyed for {}", client_id), config.clone());
}

pub async fn handle_disconnect(
    clients_ref: Arc<RwLock<HashMap<u64, SocketAddr>>>,
    client_id: u64,
    client_topics: Arc<RwLock<HashMap<u64, HashSet<u64>>>>,
    config: Arc<Config>,
) {
    /* need to clear :
    clients hashmap => will stop ping_sender task and heartbeat_checker task
        each task clear his own data
        ping_sender => clients_ping_ref
        heartbeat_checker => client_heartbeat_ref
     */

    // loops trough client_topics topics and remove the client from the topic
    let client_topics = &client_topics;
    let topic_ids: Vec<u64> = {
        let read_client_topics = client_topics.read().await;
        read_client_topics.keys().cloned().collect()
    };
    let mut write_client_topics = client_topics.write().await;
    for topic_id in topic_ids {
        write_client_topics.get_mut(&topic_id).unwrap().remove(&client_id);
    }
    clients_ref.write().await.remove(&client_id);
    log(Info, Other, format!("Disconnect {}", client_id), config.clone());
}

/**
 */
async fn topics_request_handler(
    sender: Arc<UdpSocket>,
    buffer: [u8; 1024],
    client_id: u64,
    clients: Arc<RwLock<HashMap<u64, SocketAddr>>>,
    clients_topics: Arc<RwLock<HashMap<u64, HashSet<u64>>>>,
    root_ref: Arc<RwLock<TopicV2>>,
    config: Arc<Config>,
) {
    // 1 - Init local variables
    let client_addr = *clients.read().await.get(&client_id).unwrap();
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
                }
                Err(_) => result = sender.send_to(&RQ_TopicRequest_NACK::new(TopicsResponse::FAILURE_SUB, "Subscribing to {} failed :(".to_string()).as_bytes(), client_addr).await,
            }
        }
        TopicsAction::UNSUBSCRIBE => {
            let topic_id = custom_string_hash(&topic_path);

            clients_topics.write().await.entry(topic_id).and_modify(|vec| { vec.retain(|e| *e != client_id) });

            result = sender.send_to(&RQ_TopicRequest_ACK::new(topic_id, TopicsResponse::SUCCESS_USUB).as_bytes(), client_addr).await;
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