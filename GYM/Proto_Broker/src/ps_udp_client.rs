use std::net::UdpSocket;
use uuid::Uuid;

pub struct Client {
    id: Uuid,
    socket: UdpSocket,
}

impl Client {

    pub const fn new(id: Uuid, stream: UdpSocket) -> Client {
        Client{
            id: id,
            socket: stream
        }
    }

    pub fn connect(addr : String) -> Client {
        let x = format!("0.0.0.0:{}", addr.rsplit_once(':').unwrap().1);
        println!("{}", x);
        let ret = UdpSocket::bind(x);
        match ret{
            Ok(socket) => {
                println!("connected to {addr}");
                let mut socket = socket;
                socket.connect(addr.clone()).expect("wow");
                // Todo receive le Uuid from server

                return Client::new(Uuid::new_v4(), socket);
            }
            Err(e) => {
                println!("Error connecting to {addr}");
                println!("{}", e);
                panic!()
            }
        }
    }

    pub fn send_bytes(&self, bytes: &[u8], remote_addr: &String) -> bool {
        let size = self.socket.send_to(bytes, remote_addr);
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