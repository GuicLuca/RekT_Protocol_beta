use std::collections::HashMap;
use std::io::BufRead;
use std::net::TcpListener;
use std::thread;

use crate::ps_common;

pub struct Server {
    address: String,
    port: i16,
    state: HashMap<i64, Vec<i64>>,
    listener: Option<TcpListener>,
}

impl Server {
    pub fn new(address: String, port: i16) -> Self {
        Server {
            address,
            port,
            state: Default::default(),
            listener: None,
        }
    }

    pub fn serve(&mut self) {
        let addr = format!("{}:{}", self.address, self.port);
        let ret = TcpListener::bind(addr.clone());
        match ret {
            Ok(listener) => {
                println!("Running on {addr}");
                self.listener = Some(listener);
                self.main_loop()
            }
            Err(e) => {
                println!("Error listening on {addr}");
                println!("{e}");
                panic!()
            }
        }
    }

    fn main_loop(&self) {
        loop {
            for ret in self.listener.as_ref().unwrap().incoming() {
                match ret {
                    Ok(mut stream) => {
                        println!("New connection: {}", stream.peer_addr().unwrap());
                        thread::spawn(move || {
                            // connection succeeded
                            let mut reader = std::io::BufReader::new(&mut stream);
                            loop {
                                let mut bytes: Vec<u8> = vec![];
                                match reader.read_until(b'\n', &mut bytes) {
                                    Ok(0) => {
                                        // Connection closed by the client
                                        break;
                                    }
                                    Ok(_n) => {
                                        // Read n bytes from the client
                                        println!("{}", String::from_utf8(bytes).unwrap());
                                    }
                                    Err(e) => {
                                        println!("Error: {}", e);
                                        break;
                                    }
                                }
                            }
                        });
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                        /* connection failed */
                    }
                }
            }
        }
    }
}