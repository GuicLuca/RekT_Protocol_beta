use std::net::UdpSocket;
use uuid::Uuid;
use crate::ps_datagram_structs::*;

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
        let addr = format!("0.0.0.0:{}", addr.rsplit_once(':').unwrap().1);
        println!("{}", addr);
        let ret = UdpSocket::bind(addr.clone());
        match ret{
            Ok(socket) => {
                println!("connected to {}", addr);
                let socket = socket;
                socket.connect(addr.clone()).expect("wow");

                // 1 - send Connect datagrams
                let connect_request = RQ_Connect::new();
                socket.send(&connect_request.as_bytes());
                // ... server answer with a connect_ack_STATUS
                let buffer:&mut[u8] = Default::default();
                socket.recv_from(buffer);

                let message_type = MessageType::from(*buffer.to_vec().first().unwrap());
                match message_type {
                    MessageType::CONNECT_ACK =>{
                        println!("Connect_Ack received !");
                        let is_succesfull = ConnectStatus::from(*buffer.to_vec().get(1).unwrap());
                        match is_succesfull {
                            ConnectStatus::SUCCESS => {
                                let peer_id = Uuid::from_slice_le(&buffer[2..10]).expect("The uuid is incorrect");
                                return Client::new(peer_id, socket);
                            }
                            ConnectStatus::FAILURE => {
                                let mut arr = [0u8; 2];
                                arr.copy_from_slice(&buffer[3..4]);
                                let message_size = u16::from_le_bytes(arr);
                                let index : usize = 5usize+usize::from(message_size);
                                let reason = &buffer[5usize..(index)];
                                println!("ERROR with the CONNECT Packet : {}",String::from_utf8(reason.to_vec()).expect("Can't convert reason to string"));
                            }
                        }
                    }
                    _ => {}
                }



                panic!("The client can't be constructed");

            }
            Err(e) => {
                println!("Error connecting to {}", addr);
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