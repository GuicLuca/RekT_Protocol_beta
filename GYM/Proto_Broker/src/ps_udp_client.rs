use std::net::UdpSocket;

use crate::ps_datagram_structs::*;

pub struct Client {
    id: u64,
    socket: UdpSocket,
}

impl Client {
    pub const fn new(id: u64, stream: UdpSocket) -> Client {
        Client {
            id,
            socket: stream,
        }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }
    fn create_client_from_connect_response(socket: UdpSocket, buffer: &[u8]) -> Client {
        // Get the packet type from the message type
        let message_type = MessageType::from(*buffer.to_vec().first().unwrap());
        match message_type {
            MessageType::CONNECT_ACK => {
                println!("Connect_Ack received !");
                let is_succesfull = ConnectStatus::from(*buffer.to_vec().get(1).unwrap());
                match is_succesfull {
                    ConnectStatus::SUCCESS => {
                        // Copy a part of the buffer to an array to convert it to u64
                        let mut arr = [0u8; 8];
                        arr.copy_from_slice(&buffer[2..10]);
                        let peer_id = u64::from_le_bytes(arr);
                        return Client::new(peer_id, socket);
                    }
                    ConnectStatus::FAILURE => {
                        // Copy a part of the buffer to an array to convert it to u16
                        let mut arr = [0u8; 2];
                        arr.copy_from_slice(&buffer[2..4]);
                        let message_size = u16::from_le_bytes(arr);
                        let index: usize = 5usize + usize::from(message_size);
                        let reason = &buffer[5usize..(index)];
                        println!("ERROR with the CONNECT Packet : {}", String::from_utf8(reason.to_vec()).expect("Can't convert reason to string"));
                        panic!("The client can't be constructed");
                    }
                    ConnectStatus::UNKNOWN => {
                        panic!("Unknown connect status");
                    }
                }
            }
            _ => {
                panic!("Unknown connect status response");
            }
        }
    }

    pub fn connect(addr: String) -> Client {
        let addr1 = format!("0.0.0.0:{}", addr.rsplit_once(':').unwrap().1);
        println!("{}", addr1);
        let ret = UdpSocket::bind(addr1.clone());
        match ret {
            Ok(socket) => {
                println!("connected to {}", addr);
                let socket = socket;
                socket.connect(addr.clone()).expect("wow");

                // 1 - send Connect datagrams
                let connect_request = RQ_Connect::new();
                socket.send(&connect_request.as_bytes());
                // ... server answer with a connect_ack_STATUS
                let mut buffer = [0; 1024];
                socket.recv_from(&mut buffer);
                return Client::create_client_from_connect_response(socket, &buffer);
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