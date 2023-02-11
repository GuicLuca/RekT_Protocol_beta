use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use crate::ps_datagram_structs::RQ_Connect_ACK_OK;

pub struct Server {
    address: String,
    port: i16,
    state: HashMap<i64, Vec<i64>>,
    socket: UdpSocket,
    hashmap : HashMap<u64,SocketAddr>,
    last_id : u64
}

impl Server {
    pub fn new(address: String, port: i16, socket: UdpSocket) -> Self {
        Server {
            address,
            port,
            state: Default::default(),
            socket,
            hashmap : HashMap::new(),
            last_id : 0,
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
    fn get_new_id(&mut self) -> u64{
        let id = self.last_id+1;
        self.last_id = id;
        return id;
    }
    pub fn main_loop(&mut self) {

        loop {
            let mut buf = [0; 1024];
            match self.socket.recv_from(&mut buf) {
                Ok((n, src)) => {
                    println!("Received {} bytes from {}", n, src);
                    let uuid = self.get_new_id();
                    self.hashmap.insert(uuid,src);
                    let RQ_ConnectACK = RQ_Connect_ACK_OK::new(uuid,1);
                    let result = self.socket.send_to(&RQ_ConnectACK.as_bytes(),src);
                    println!("Send {} bytes",result.unwrap());
                    //println!("{}", String::from_utf8_lossy(&buf[..n]));
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
    }
}

