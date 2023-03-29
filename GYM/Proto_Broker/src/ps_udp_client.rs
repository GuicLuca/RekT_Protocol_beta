use std::net::UdpSocket;

use crate::ps_datagram_structs::*;
use crate::ps_common::*;

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
                let is_successful = ConnectStatus::from(*buffer.to_vec().get(1).unwrap());
                match is_successful {
                    ConnectStatus::SUCCESS => {

                        // Copy a part of the buffer to an array to convert it to u64
                        let peer_id = u64::from_le_bytes(buffer[2..10].to_vec().try_into().unwrap());
                        println!("New client : {}", peer_id);
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
    pub fn create_topic_test(&self) -> std::io::Result<usize> {
        let topic_rq = RQ_TopicRequest::new(TopicsAction::SUBSCRIBE, "/home/topix/xd");
        self.socket.send(&topic_rq.as_bytes())
    }

    pub fn connect(addr: String) -> Client {
        let addr1 = format!("0.0.0.0:{}", addr.rsplit_once(':').unwrap().1);
        println!("{}", addr1);
        let ret = UdpSocket::bind("0.0.0.0:3939");
        match ret {
            Ok(socket) => {
                println!("connected to {}", addr);
                let socket = socket;
                socket.connect(addr.clone()).expect("wow");

                // 1 - send Connect datagrams
                let connect_request = RQ_Connect::new();
                socket.send(&connect_request.as_bytes()).expect("TODO: panic message");
                // ... server answer with a connect_ack_STATUS
                let mut buffer = [0; 1024];
                socket.recv_from(&mut buffer).expect("TODO: panic message");
                return Client::create_client_from_connect_response(socket, &buffer);
            }
            Err(e) => {
                println!("Error connecting to {}", addr);
                println!("{}", e);
                panic!()
            }
        }
    }

    pub fn wait_xd(&self) {
        loop {
            let mut buffer = [0; 1024];
            // 2 - Wait for bytes reception
            match self.socket.recv_from(&mut buffer) {
                // 3 - Once bytes are received, check for errors
                Ok((n, src)) => {
                    // 4 - if OK match on the first byte (MESSAGE_TYPE)
                    println!("[Client Handler] Received {} bytes from server({})", n, src);

                    match MessageType::from(buffer[0]) {
                        MessageType::CONNECT => {}
                        MessageType::DATA => {}
                        MessageType::OPEN_STREAM => {}
                        MessageType::SHUTDOWN => {}
                        MessageType::HEARTBEAT => {
                            //self.socket.send_to(&RQ_Heartbeat::new().as_bytes(), src);
                        }
                        MessageType::OBJECT_REQUEST => {}
                        MessageType::TOPIC_REQUEST => {}
                        MessageType::PING => {
                            // 5.x - Send a immediate response to the server
                            let result = self.socket.send_to(&RQ_Pong::new(buffer[1]).as_bytes(), src);
                            match result {
                                Ok(bytes) => {
                                    println!("[Client - Ping] Send {} bytes to server({})", bytes, src.ip());
                                }
                                Err(_) => {
                                    println!("[Client - Ping] Failed to send pong to server({})", src.ip());
                                }
                            }
                        }
                        MessageType::TOPIC_REQUEST_ACK | MessageType::OBJECT_REQUEST_ACK | MessageType::CONNECT_ACK | MessageType::PONG => {}
                        MessageType::UNKNOWN => {
                            println!("[Client] Received unknown packet from {}", src.ip())
                        }
                        MessageType::HEARTBEAT_REQUEST => {
                            println!("[Client - Heart beat] Received an HB request so answer to it !");
                            self.socket.send_to(&RQ_Heartbeat::new().as_bytes(), src);
                        }
                    }
                    //192.168.0.180
                   for i in 0..2  {
                        self.socket.send_to(&RQ_TopicRequest::new(TopicsAction::SUBSCRIBE, "/home/topix/xd/ez/ez/ez/ez/ez/ez/ez/ez/ez").as_bytes(), src);
                        //self.socket.send_to(&RQ_Heartbeat::new().as_bytes(), src);
                   }
                    //println!("{}", String::from_utf8_lossy(&buf[..n]));
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
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