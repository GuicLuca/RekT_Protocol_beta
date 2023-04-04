use std::sync::Arc;
use std::time::Duration;
use rand::distributions::Alphanumeric;
use rand::Rng;
use tokio::net::UdpSocket;
use tokio::time::MissedTickBehavior::Delay;
use tokio::time::sleep;

use crate::ps_datagram_structs::*;
use crate::ps_common::*;

pub struct Client {
    id: u64,
    socket: Arc<UdpSocket>,
    topic_id:u64,
}

impl Client {
    pub fn new(id: u64, stream: UdpSocket) -> Client {
        Client {
            id,
            socket: Arc::new(stream),
            topic_id:0
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
    pub async fn create_topic_test(&self) -> std::io::Result<usize> {
        let topic_rq = RQ_TopicRequest::new(TopicsAction::SUBSCRIBE, "/home/topix/xd");
        self.socket.send(&topic_rq.as_bytes()).await
    }

    pub async fn connect(addr: String) -> Client {
        let addr1 = format!("0.0.0.0:{}", addr.rsplit_once(':').unwrap().1);
        println!("{}", addr1);
        let ret = UdpSocket::bind("0.0.0.0:3939").await;
        match ret {
            Ok(socket) => {
                println!("connected to {}", addr);
                let socket = socket;
                socket.connect(addr.clone()).await;

                // 1 - send Connect datagrams
                let connect_request = RQ_Connect::new();
                socket.send(&connect_request.as_bytes()).await;
                // ... server answer with a connect_ack_STATUS
                let mut buffer = [0; 1024];
                socket.recv_from(&mut buffer).await;
                return Client::create_client_from_connect_response(socket, &buffer);
            }
            Err(e) => {
                println!("Error connecting to {}", addr);
                println!("{}", e);
                panic!()
            }
        }
    }

    pub async fn wait_xd(&mut self) {
        let mut sequence_number = 0;
        self.create_topic_test();
        let random_msg: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(500)
            .map(char::from)
            .collect();

        let socket = self.socket.clone();
        tokio::spawn(async move{
            while true {
                socket.send(&RQ_TopicRequest::new(TopicsAction::SUBSCRIBE, "/ez/EZ/EZ/EZ/EZ/EZ/EZ/EZ/EZ/EZ").as_bytes()).await;
                sequence_number +=1;
            }
        });

        loop {
            let mut buffer = [0; 1024];
            // 2 - Wait for bytes reception
            match self.socket.recv_from(&mut buffer).await {
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
                            let result = self.socket.send_to(&RQ_Pong::new(buffer[1]).as_bytes(), src).await;
                            match result {
                                Ok(bytes) => {
                                    println!("[Client - Ping] Send {} bytes to server({})", bytes, src.ip());
                                }
                                Err(_) => {
                                    println!("[Client - Ping] Failed to send pong to server({})", src.ip());
                                }
                            }
                        }
                        MessageType::TOPIC_REQUEST_ACK =>{
                            if(TopicsResponse::from(buffer[1]) == TopicsResponse::SUCCESS_SUB){
                                println!("{:?}", buffer);
                                self.topic_id = RQ_TopicRequest_ACK::from(buffer.as_ref()).topic_id;
                            }
                        }
                        MessageType::TOPIC_REQUEST_NACK | MessageType::OBJECT_REQUEST_ACK | MessageType::CONNECT_ACK | MessageType::PONG => {}
                        MessageType::UNKNOWN => {
                            println!("[Client] Received unknown packet from {}", src.ip())
                        }
                        MessageType::HEARTBEAT_REQUEST => {
                            println!("[Client - Heart beat] Received an HB request so answer to it !");
                            self.socket.send_to(&RQ_Heartbeat::new().as_bytes(), src);
                        }
                    }
                    //192.168.0.180
                    /*if(self.topic_id != 0){
                        for i in 0..100  {
                            println!("sent data");
                            self.socket.send_to(&RQ_Data::new(sequence_number,self.topic_id, random_msg.as_bytes().to_vec()).as_bytes(), src);
                            sequence_number +=1;
                        }
                    }*/
                    //println!("{}", String::from_utf8_lossy(&buf[..n]));
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
    }

    pub async fn send_bytes(&self, bytes: &[u8], remote_addr: &String) -> bool {
        let size = self.socket.send_to(bytes, remote_addr).await;
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