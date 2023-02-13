use std::collections::HashMap;
use std::io::ErrorKind;

use std::net::{IpAddr, SocketAddr, UdpSocket};

use rand::Rng;
use try_catch::catch;

use crate::ps_common::string_to_hash;
use crate::ps_datagram_structs::{MessageType, RQ_Connect_ACK_ERROR, RQ_Connect_ACK_OK};
use crate::topic::*;

pub struct Server {
    address: String,
    port: i16,
    topics: HashMap<u64, Vec<u64>>,
    socket: UdpSocket,
    clients: HashMap<u64, SocketAddr>,
    last_id: u64,
    root: Topic,
}

impl Server {
    pub fn new(address: String, port: i16, socket: UdpSocket) -> Self {
        Server {
            address,
            port,
            topics: Default::default(),
            root: Topic::new(1),
            socket,
            clients: HashMap::new(),
            last_id: 0,
        }
    }

    pub fn serve(address: String, port: i16) -> Server {
        let addr = format!("{}:{}", address, port);
        let ret = UdpSocket::bind(addr.clone());
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
            // bc we love recursivity
            nb = *&self.get_new_id();
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

    pub fn create_topics(&mut self, payload: String) {
        let it = payload.split("/");
        let vec: Vec<&str> = it.collect();
        let mut last_created_topic = &mut self.root;
        let i = 0;
        while {
            let mut hash: String = String::from("/");
            for j in 0..i {
                hash.push_str(&vec[j].to_string());
                hash.push_str("/");
            }
            let id = string_to_hash(&vec[i].to_string());
            let new_topic = Topic::new(id);
            last_created_topic.add_sub_topic(new_topic);

            last_created_topic = last_created_topic.get_sub_topic_by_id(id).expect("Topic was created but can't found");

            i < vec.len()
        } { /*do while syntax xd*/ }
    }

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
            match result  {
                Ok(bytes) => {
                    println!("Send {} bytes", bytes);
                }
                Err(_) => {
                    println!("Failed to send Connect ACK to {}", src);
                }
        }
    }

    pub fn main_loop(&mut self) {
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
                            self.create_topics("/home/xd/2".to_string())
                        }
                        MessageType::PING => {}
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
    }
}

