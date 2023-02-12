use rand::Rng;
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};

use crate::ps_datagram_structs::{MessageType, RQ_Connect_ACK_OK};

pub struct Server {
    address: String,
    port: i16,
    state: HashMap<i64, Vec<i64>>,
    socket: UdpSocket,
    hashmap: HashMap<u64, SocketAddr>,
    last_id: u64,
}

impl Server {
    pub fn new(address: String, port: i16, socket: UdpSocket) -> Self {
        Server {
            address,
            port,
            state: Default::default(),
            socket,
            hashmap: HashMap::new(),
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
        if self.hashmap.contains_key(&nb) {
            // bc we love recursivity
            nb = *&self.get_new_id();
        }
        return nb;
    }

    pub fn invalid_msg_type(&self, src: &SocketAddr) {
        println!("recieved invalid packet from {}", src.ip())
    }

    pub fn main_loop(&mut self) {
        loop {
            let mut buf = [0; 1024];
            match self.socket.recv_from(&mut buf) {
                Ok((n, src)) => {
                    println!("Received {} bytes from {}", n, src);
                    match MessageType::from(buf[0]) {
                        MessageType::CONNECT => {
                            let uuid = self.get_new_id();
                            self.hashmap.insert(uuid, src);
                            let RQ_ConnectACK = RQ_Connect_ACK_OK::new(uuid, 1);
                            let result = self.socket.send_to(&RQ_ConnectACK.as_bytes(), src);
                            println!("Send {} bytes", result.unwrap());
                        }
                        MessageType::DATA => {}
                        MessageType::OPEN_STREAM => {}
                        MessageType::HEARTBEAT => {}
                        MessageType::OBJECT_REQUEST => {}
                        MessageType::TOPIC_REQUEST => {}
                        MessageType::PING => {}
                        MessageType::TOPIC_REQUEST_ACK | MessageType::OBJECT_REQUEST_ACK | MessageType::CONNECT_ACK | MessageType::HEARTBEAT_REQUEST | MessageType::SHUTDOWN | MessageType::PONG => {
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

