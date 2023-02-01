use std::collections::HashMap;
use std::net::UdpSocket;



pub struct Server {
    address: String,
    port: i16,
    state: HashMap<i64, Vec<i64>>,
    socket: Option<UdpSocket>,
}

impl Server {
    pub fn new(address: String, port: i16) -> Self {
        Server {
            address,
            port,
            state: Default::default(),
            socket: None,
        }
    }

    pub fn serve(&mut self) {
        let addr = format!("{}:{}", self.address, self.port);
        let ret = UdpSocket::bind(addr.clone());
        match ret {
            Ok(socket) => {
                println!("Running on {}", addr);
                self.socket = Some(socket);
                self.main_loop()
            }
            Err(e) => {
                println!("Error binding to {}", addr);
                println!("{}", e);
                panic!()
            }
        }
    }

    fn main_loop(&self) {
        loop {
            let mut buf = [0; 1024];
            match self.socket.as_ref().unwrap().recv_from(&mut buf) {
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

