use std::collections::HashMap;
use std::net::UdpSocket;

pub struct Server {
    address: String,
    port: i16,
    state: HashMap<i64, Vec<i64>>,
    socket: UdpSocket,
}

impl Server {
    pub fn new(address: String, port: i16, socket: UdpSocket) -> Self {
        Server {
            address,
            port,
            state: Default::default(),
            socket,
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

    pub fn main_loop(&self) {
        loop {
            let mut buf = [0; 1024];
            match self.socket.recv_from(&mut buf) {
                Ok((n, src)) => {
                    println!("Received {} bytes from {}", n, src);
                    println!("{}", String::from_utf8_lossy(&buf[..n]));
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
    }
}

