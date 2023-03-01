use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, ErrorKind,};
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rand::Rng;
use try_catch::catch;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf},
    net::UdpSocket,
};

use crate::ps_common::string_to_hash;
use crate::ps_datagram_structs::{MessageType, RQ_Connect_ACK_ERROR, RQ_Connect_ACK_OK, RQ_Ping, RQ_TopicRequest_ACK, TopicsResponse};

use crate::topic_v2::TopicV2;

pub struct Server {
    address: String,
    port: i16,
    topics: HashMap<u64, Vec<u64>>,
    reader: ReadHalf<tokio::net::UdpSocket>,
    writer: WriteHalf<tokio::net::UdpSocket>,
    clients: HashMap<u64, SocketAddr>,
    root: TopicV2,
    b_running : bool,
    clients_ping: HashMap<u64, SocketAddr>,
    pings: HashMap<u8, u128>, // <ping_id, time when the ping has been made
}

impl Server {
    pub fn new(address: String, port: i16,&mut socket: &mut tokio::net::UdpSocket) -> Self {
        //let (reader, mut writer) = socket.split();
        Server {
            address,
            port,
            topics: Default::default(),
            root: TopicV2::new(1),
            reader,
            writer,
            clients: HashMap::new(),
            b_running: false,
            clients_ping: HashMap::new(),
            pings: HashMap::new(),
        }
    }

    // - done
    pub fn serve(address: String, port: i16) -> Server {
        let addr = format!("{}:{}", address, port);
        let &mut ret = UdpSocket::bind(addr.clone());
        match ret {
            Ok(socket) => {
                println!("Running on {}", addr);
                return Self::new(address, port, socket);
            }
            Err(e) => {
                println!("Error binding to {}", addr);
                println!("{}", e);
                panic!()
            }
        }
    }

    fn get_new_id(&mut self) -> u64 {
        let mut rng = rand::thread_rng();
        let mut nb: u64 = rng.gen();
        if self.clients.contains_key(&nb) {
            // bc we love recursive
            nb = *&self.get_new_id();
        }
        return nb;
    }

    fn get_new_ping_id(& self) -> u8 {
        let mut rng = rand::thread_rng();
        let mut nb: u8 = rng.gen();
        if self.pings.contains_key(&nb) {
            // bc we love recursive
            nb = self.get_new_ping_id();
        }
        return nb;
    }

    pub fn invalid_msg_type(&self, src: &SocketAddr) {
        println!("recieved invalid packet from {}", src.ip())
    }

    fn already_connected(&self, ip: &IpAddr) -> (bool, u64) {
        for (key, value) in &self.clients {
            if ip == &value.ip() {
                return (true, *key);
            }
        }
        return (false, 0);
    }


    /*pub fn create_topics(&mut self, path: &str) -> u64 {
        TopicV2::create_topics(path, &mut self.root)
    }*/


    /*
        pub fn create_topics(&mut self, path: &str) -> u64 {
            let mut last_created_topic_id = self.root.id;
            let mut current_topic = &mut self.root;
            let topic_names: Vec<&str> = path.split("/").collect();
            for topic_name in topic_names {
                if topic_name.is_empty() {
                    continue;
                }
                let topic_id = {
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    topic_name.hash(&mut hasher);
                    hasher.finish()
                };
                match current_topic.sub_topics.get_mut(&topic_id) {
                    Some(existing_topic) => {
                        current_topic = existing_topic;
                    }
                    None => {
                        let mut new_topic = TopicV2::new(topic_id);
                        current_topic.add_sub_topic(new_topic.clone());
                        current_topic = current_topic.sub_topics.get_mut(&topic_id).unwrap();
                    }
                }
                last_created_topic_id = topic_id;
            }
            last_created_topic_id
        }

        pub fn create_topics(&mut self, payload: String) -> u64 {
            let vec: Vec<&str> = payload.split("/").collect();

            let mut hash = String::from("");
            let mut last_created_topic = &mut self.root;
            for i in 1..vec.len() {
                hash.push_str("/");
                hash.push_str(vec[i]);

                let id = string_to_hash(&hash);
                let created_topic = last_created_topic.get_sub_topic_by_id(id);
                if let Some(topic) = created_topic {
                    last_created_topic = topic;
                } else {
                    let new_topic = Topic::new(id);
                    last_created_topic.add_sub_topic(new_topic);
                    last_created_topic = last_created_topic.get_sub_topic_by_id(id).expect("Topic was created but cannot be found");
                }
            }
            last_created_topic.get_id()
        }*/


    fn handle_connect(&mut self, src: SocketAddr) {
        let (is_connected, current_id) = self.already_connected(&src.ip());
        let uuid;
        let result;
        if is_connected {
            uuid = current_id;
        } else {
            uuid = self.get_new_id();
            self.clients.insert(uuid, src);
        }
        result = self.socket.send_to(&RQ_Connect_ACK_OK::new(uuid, 1).as_bytes(), src);
        match result {
            Ok(bytes) => {
                println!("Send {} bytes", bytes);
            }
            Err(_) => {
                println!("Failed to send Connect ACK to {}", src);
            }
        }
    }

    /*fn send_ping_request(&mut self) -> JoinHandle<()>
    {
        //Start a new thread to send periodically ping request to clients
        let server = Arc::new(Mutex::new(slef));
        return thread::spawn(move || {
            unsafe {
                while self.b_running {
                    for (id, addr) in self.clients.clone() {
                        // 1 - get the current time :
                        let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                        let ping_id = self.get_new_ping_id();
                        self.pings.insert(ping_id, start);

                        self.socket.send_to(&RQ_Ping::new(ping_id).as_bytes(), addr);
                    }
                    // wait 10 sec before request again the round trip threshold
                    thread::sleep(Duration::from_secs(10));
                }
            }
        });
    }*/

    pub fn main_loop(&mut self) {
        self.b_running = true;

        loop {
            let mut buf = [0; 1024];
            match self.socket.recv_from(&mut buf) {
                Ok((n, src)) => {
                    println!("Received {} bytes from {}", n, src);
                    match MessageType::from(buf[0]) {
                        MessageType::CONNECT => {
                            self.handle_connect(src)
                        }
                        MessageType::DATA => {}
                        MessageType::OPEN_STREAM => {}
                        MessageType::SHUTDOWN => {}
                        MessageType::HEARTBEAT => {}
                        MessageType::OBJECT_REQUEST => {}
                        MessageType::TOPIC_REQUEST => {
                        /*    let topic_path = String::from_utf8(buf[1..].to_vec()).unwrap();

                            let topic_id = self.create_topics(&topic_path);

                            let result = self.socket.send_to(&RQ_TopicRequest_ACK::new(TopicsResponse::SUCCESS, topic_id).as_bytes(), src);
                            match result {
                                Ok(bytes) => {
                                    println!("Send {} bytes", bytes);
                                }
                                Err(_) => {
                                    println!("Failed to send ACK to {}", src);
                                }
                            }
                        */}
                        MessageType::PONG => {}
                        MessageType::PING => {/*Drop paquet*/}
                        MessageType::TOPIC_REQUEST_ACK | MessageType::OBJECT_REQUEST_ACK | MessageType::CONNECT_ACK | MessageType::HEARTBEAT_REQUEST | MessageType::PONG => {
                            self.invalid_msg_type(&src)
                        }
                        MessageType::UNKNOWN => {
                            println!("recieved unknown packet from {}", src.ip())
                        }
                    }

                    //println!("{}", String::from_utf8_lossy(&buf[..n]));
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
        self.b_running = false;
    }
}

