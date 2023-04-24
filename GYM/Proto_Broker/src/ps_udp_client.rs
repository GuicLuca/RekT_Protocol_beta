use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::mem::size_of;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::distributions::Alphanumeric;
use rand::Rng;
use tokio::net::UdpSocket;

use crate::ps_datagram_structs::*;

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
        let mut hasher = DefaultHasher::new();
        "/home/topix/xd".hash(&mut hasher);
        let topic_id: u64 = hasher.finish();
        let mut topic_rq = RQ_TopicRequest::new(TopicsAction::SUBSCRIBE, topic_id);
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
        /*self.create_topic_test();
        let random_msg: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(1000)
            .map(char::from)
            .collect();

        let socket = self.socket.clone();
        tokio::spawn(async move{
            loop {
                let mut hasher = DefaultHasher::new();
                "/home/topix/xd".hash(&mut hasher);
                let topic_id: u64 = hasher.finish();
                socket.send(&RQ_TopicRequest::new(TopicsAction::SUBSCRIBE, topic_id).as_bytes()).await;
                sequence_number +=1;
            }
        });

        let mut hasher = DefaultHasher::new();
        "/home/topix/xd".hash(&mut hasher);
        let mut topics: HashSet<u64> = HashSet::with_capacity(size_of::<u64>() * 4);
        topics.insert(hasher.finish());
        "/home/topix/xdjghfhjgfj".hash(&mut hasher);
        topics.insert(hasher.finish());

        self.socket.send(
            &RQ_ObjectRequest::new(ObjectFlags::CREATE, generate_object_id(ObjectIdentifierType::USER_GENERATED),topics).as_bytes()
        ).await;*/

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
                            println!("[Client - topic ack] RECIEVE A TOPIC REQUEST ACK     ___     server({})", src.ip());
                            /*if(TopicsResponse::from(buffer[1]) == TopicsResponse::SUCCESS_SUB){
                                println!("{:?}", buffer);
                                self.topic_id = RQ_TopicRequest_ACK::from(buffer.as_ref()).topic_id;

                                let socket = self.socket.clone();
                                let topic_id = self.topic_id;
                                let msg = random_msg.clone();
                                tokio::spawn(async move{
                                    loop {
                                        socket.send(&RQ_Data::new(sequence_number, topic_id, msg.as_bytes().to_vec()).as_bytes()).await;
                                        sequence_number +=1;
                                    }
                                });
                                let socket = self.socket.clone();
                                let topic_id = self.topic_id;
                                let msg = random_msg.clone();
                                tokio::spawn(async move{
                                    loop {
                                        socket.send(&RQ_Data::new(sequence_number, topic_id, msg.as_bytes().to_vec()).as_bytes()).await;
                                        sequence_number +=1;
                                    }
                                });
                                let socket = self.socket.clone();
                                let topic_id = self.topic_id;
                                let msg = random_msg.clone();
                                tokio::spawn(async move{
                                    loop {
                                        socket.send(&RQ_Data::new(sequence_number, topic_id, msg.as_bytes().to_vec()).as_bytes()).await;
                                        sequence_number +=1;
                                    }
                                });
                                let socket = self.socket.clone();
                                let topic_id = self.topic_id;
                                let msg = random_msg.clone();
                                tokio::spawn(async move{
                                    loop {
                                        socket.send(&RQ_Data::new(sequence_number, topic_id, msg.as_bytes().to_vec()).as_bytes()).await;
                                        sequence_number +=1;
                                    }
                                });
                                let socket = self.socket.clone();
                                let topic_id = self.topic_id;
                                let msg = random_msg.clone();
                                tokio::spawn(async move{
                                    loop {
                                        socket.send(&RQ_Data::new(sequence_number, topic_id, msg.as_bytes().to_vec()).as_bytes()).await;
                                        sequence_number +=1;
                                    }
                                });
                                let socket = self.socket.clone();
                                let topic_id = self.topic_id;
                                let msg = random_msg.clone();
                                tokio::spawn(async move{
                                    loop {
                                        socket.send(&RQ_Data::new(sequence_number, topic_id, msg.as_bytes().to_vec()).as_bytes()).await;
                                        sequence_number +=1;
                                    }
                                });
                                let socket = self.socket.clone();
                                let topic_id = self.topic_id;
                                let msg = random_msg.clone();
                                tokio::spawn(async move{
                                    loop {
                                        socket.send(&RQ_Data::new(sequence_number, topic_id, msg.as_bytes().to_vec()).as_bytes()).await;
                                        sequence_number +=1;
                                    }
                                });
                                let socket = self.socket.clone();
                                let topic_id = self.topic_id;
                                let msg = random_msg.clone();
                                tokio::spawn(async move{
                                    loop {
                                        socket.send(&RQ_Data::new(sequence_number, topic_id, msg.as_bytes().to_vec()).as_bytes()).await;
                                        sequence_number +=1;
                                    }
                                });
                                let socket = self.socket.clone();
                                let topic_id = self.topic_id;
                                let msg = random_msg.clone();
                                tokio::spawn(async move{
                                    loop {
                                        socket.send(&RQ_Data::new(sequence_number, topic_id, msg.as_bytes().to_vec()).as_bytes()).await;
                                        sequence_number +=1;
                                    }
                                });
                                let socket = self.socket.clone();
                                let topic_id = self.topic_id;
                                let msg = random_msg.clone();
                                tokio::spawn(async move{
                                    loop {
                                        socket.send(&RQ_Data::new(sequence_number, topic_id, msg.as_bytes().to_vec()).as_bytes()).await;
                                        sequence_number +=1;
                                    }
                                });
                                let socket = self.socket.clone();
                                let topic_id = self.topic_id;
                                let msg = random_msg.clone();
                                tokio::spawn(async move{
                                    loop {
                                        socket.send(&RQ_Data::new(sequence_number, topic_id, msg.as_bytes().to_vec()).as_bytes()).await;
                                        sequence_number +=1;
                                    }
                                });
                                let socket = self.socket.clone();
                                let topic_id = self.topic_id;
                                let msg = random_msg.clone();
                                tokio::spawn(async move{
                                    loop {
                                        socket.send(&RQ_Data::new(sequence_number, topic_id, msg.as_bytes().to_vec()).as_bytes()).await;
                                        sequence_number +=1;
                                    }
                                });
                                let socket = self.socket.clone();
                                let topic_id = self.topic_id;
                                let msg = random_msg.clone();
                                tokio::spawn(async move{
                                    loop {
                                        socket.send(&RQ_Data::new(sequence_number, topic_id, msg.as_bytes().to_vec()).as_bytes()).await;
                                        sequence_number +=1;
                                    }
                                });
                                let socket = self.socket.clone();
                                let topic_id = self.topic_id;
                                let msg = random_msg.clone();
                                tokio::spawn(async move{
                                    loop {
                                        socket.send(&RQ_Data::new(sequence_number, topic_id, msg.as_bytes().to_vec()).as_bytes()).await;
                                        sequence_number +=1;
                                    }
                                });
                                let socket = self.socket.clone();
                                let topic_id = self.topic_id;
                                let msg = random_msg.clone();
                                tokio::spawn(async move{
                                    loop {
                                        socket.send(&RQ_Data::new(sequence_number, topic_id, msg.as_bytes().to_vec()).as_bytes()).await;
                                        sequence_number +=1;
                                    }
                                });
                                let socket = self.socket.clone();
                                let topic_id = self.topic_id;
                                let msg = random_msg.clone();
                                tokio::spawn(async move{
                                    loop {
                                        socket.send(&RQ_Data::new(sequence_number, topic_id, msg.as_bytes().to_vec()).as_bytes()).await;
                                        sequence_number +=1;
                                    }
                                });
                                let socket = self.socket.clone();
                                let topic_id = self.topic_id;
                                let msg = random_msg.clone();
                                tokio::spawn(async move{
                                    loop {
                                        socket.send(&RQ_Data::new(sequence_number, topic_id, msg.as_bytes().to_vec()).as_bytes()).await;
                                        sequence_number +=1;
                                    }
                                });
                                let socket = self.socket.clone();
                                let topic_id = self.topic_id;
                                let msg = random_msg.clone();
                                tokio::spawn(async move{
                                    loop {
                                        socket.send(&RQ_Data::new(sequence_number, topic_id, msg.as_bytes().to_vec()).as_bytes()).await;
                                        sequence_number +=1;
                                    }
                                });
                                let socket = self.socket.clone();
                                let topic_id = self.topic_id;
                                let msg = random_msg.clone();
                                tokio::spawn(async move{
                                    loop {
                                        socket.send(&RQ_Data::new(sequence_number, topic_id, msg.as_bytes().to_vec()).as_bytes()).await;
                                        sequence_number +=1;
                                    }
                                });
                                let socket = self.socket.clone();
                                let topic_id = self.topic_id;
                                let msg = random_msg.clone();
                                tokio::spawn(async move{
                                    loop {
                                        socket.send(&RQ_Data::new(sequence_number, topic_id, msg.as_bytes().to_vec()).as_bytes()).await;
                                        sequence_number +=1;
                                    }
                                });
                                let socket = self.socket.clone();
                                let topic_id = self.topic_id;
                                let msg = random_msg.clone();
                                tokio::spawn(async move{
                                    loop {
                                        socket.send(&RQ_Data::new(sequence_number, topic_id, msg.as_bytes().to_vec()).as_bytes()).await;
                                        sequence_number +=1;
                                    }
                                });
                                let socket = self.socket.clone();
                                let topic_id = self.topic_id;
                                let msg = random_msg.clone();
                                tokio::spawn(async move{
                                    loop {
                                        socket.send(&RQ_Data::new(sequence_number, topic_id, msg.as_bytes().to_vec()).as_bytes()).await;
                                        sequence_number +=1;
                                    }
                                });
                                let socket = self.socket.clone();
                                let topic_id = self.topic_id;
                                let msg = random_msg.clone();
                                tokio::spawn(async move{
                                    loop {
                                        socket.send(&RQ_Data::new(sequence_number, topic_id, msg.as_bytes().to_vec()).as_bytes()).await;
                                        sequence_number +=1;
                                    }
                                });
                                let socket = self.socket.clone();
                                let topic_id = self.topic_id;
                                let msg = random_msg.clone();
                                tokio::spawn(async move{
                                    loop {
                                        socket.send(&RQ_Data::new(sequence_number, topic_id, msg.as_bytes().to_vec()).as_bytes()).await;
                                        sequence_number +=1;
                                    }
                                });
                                let socket = self.socket.clone();
                                let topic_id = self.topic_id;
                                let msg = random_msg.clone();
                                tokio::spawn(async move{
                                    loop {
                                        socket.send(&RQ_Data::new(sequence_number, topic_id, msg.as_bytes().to_vec()).as_bytes()).await;
                                        sequence_number +=1;
                                    }
                                });
                            }*/
                        }
                        MessageType::SERVER_STATUS | MessageType::TOPIC_REQUEST_NACK | MessageType::CONNECT_ACK | MessageType::PONG => {}
                        MessageType::UNKNOWN => {
                            println!("[Client] Received unknown packet from {}", src.ip())
                        }
                        MessageType::HEARTBEAT_REQUEST => {
                            println!("[Client - Heart beat] Received an HB request so answer to it !");
                            self.socket.send_to(&RQ_Heartbeat::new().as_bytes(), src);
                        }
                        MessageType::OBJECT_REQUEST_NACK => {
                            println!("[Client - OBJ request nack] Received an object request NACK response : {:?}!", buffer);
                        }
                        MessageType::OBJECT_REQUEST_ACK => {
                            println!("[Client - OBJ request ack] Received an object request ACK response : {:?}!", buffer);
                            self.socket.send(
                                &RQ_ObjectRequest::new(ObjectFlags::DELETE, u64::from_le_bytes(buffer[10..].split_at(size_of::<u64>()).0.try_into().unwrap()), HashSet::default()).as_bytes()
                            ).await;
                        }
                        MessageType::SERVER_STATUS_ACK => {
                            println!("[Client - server_status ack] Received an server status request ACK response : {:?}!", buffer);
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

/**
 * This method generate a new object id and set
 * the two MSB according to the type needed.
 * /!\ Unknown type is not allow and will return 0.
 *
 * @param id_type: ObjectIdentifierType
 *
 * @return ObjectId or 0
 */
pub fn generate_object_id(id_type: ObjectIdentifierType) -> ObjectId {
    // 1 - Get the current time
    let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Failed to get system time").as_nanos() as u64;
    let u64_with_msb_00 = now & 0x3FFFFFFFFFFFFFFF; // the mask allow to set two MSB to 00 to rewrite them after
    // 2 - set é MSB according to the type
    match id_type {
        ObjectIdentifierType::USER_GENERATED => {
            u64_with_msb_00 | 0x0000000000000000
        }
        ObjectIdentifierType::BROKER_GENERATED => {
            u64_with_msb_00 | 0x0100000000000000
        }
        ObjectIdentifierType::TEMPORARY => {
            u64_with_msb_00 | 0x1000000000000000
        }
        _ => 0
    }
}