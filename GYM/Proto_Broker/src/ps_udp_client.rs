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
        let addr = format!("{}:{}", ip.trim(), port.trim());
        let ret = UdpSocket::bind(format!("0.0.0.0:10201"));
        match ret{
            Ok(socket) => {
                println!("connected to {addr}");
                self.socket = Some(socket);
                self.socket.as_ref().unwrap().connect(addr.clone()).expect("wow");

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