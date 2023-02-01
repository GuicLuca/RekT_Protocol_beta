use std::io::Write;
use std::net::TcpStream;

use uuid::Uuid;
use crate::ps_common;

pub struct Client {
    id: Option<Uuid>,
    stream: Option<TcpStream>,
}

impl Client {

    pub const fn new() -> Client {
        Client{
            id: None,
            stream: None
        }
    }

    pub fn connect(&mut self, ip: &String, port: &String) {
        let mut addr = ip.clone();
        addr.push_str(":");
        addr.push_str(port);
        let ret = TcpStream::connect(addr.clone());
        match ret {
            Ok(stream) => {
                println!("connected to {addr}");
                self.stream = Some(stream);
            }
            Err(e) => {
                println!("Error connecting to {addr}");
                println!("{}", e);
                panic!()
            }
        }
    }

    pub fn send_bytes(&self, bytes: &[u8]) -> bool{
        let size = self.stream.unwrap().write(bytes);
        if size.is_err() {
            println!("{}", size.unwrap_err());
            return false;
        }
        let mut percentage = 0;
        if bytes.len() > 0 {
            percentage = (size.unwrap() / bytes.len()) * 100;
        }
        println!("{percentage}% bytes sent");
        return true;
    }

}