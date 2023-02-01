use std::net::UdpSocket;
use uuid::Uuid;

pub struct Client {
    id: Option<Uuid>,
    socket: Option<UdpSocket>,
}

impl Client {

    pub const fn new() -> Client {
        Client{
            id: None,
            socket: None
        }
    }

    pub fn connect(&mut self, ip: &String, port: &String) {
        let mut addr = ip.clone();
        addr.push_str(":");
        addr.push_str(port);
        let ret = UdpSocket::bind(addr.clone());
        match ret {
            Ok(socket) => {
                println!("connected to {addr}");
                self.socket = Some(socket);
            }
            Err(e) => {
                println!("Error connecting to {addr}");
                println!("{}", e);
                panic!()
            }
        }
    }

    pub fn send_bytes(&self, bytes: &[u8], remote_addr: &String) -> bool {
        let socket = self.socket.as_ref().unwrap();
        let size = socket.send_to(bytes, remote_addr);
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