use tokio::{
   net::{UdpSocket},
};
use std::collections::HashMap;
use std::net::{SocketAddr};
use std::sync::Arc;
use local_ip_address::local_ip;
use tokio::sync::Mutex;

use crate::topic_v2::TopicV2;

mod ps_datagram_structs;
mod topic_v2;
mod ps_common;
mod ps_server_lib;
use crate::ps_datagram_structs::*;
use crate::ps_server_lib::*;

#[tokio::main]
async fn main(){
   println!("[Server] Hi there ! Chose the port of for the server :");
   let port = ps_common::get_cli_input("Input port : ", "Cannot get the port form the cli input.", None, None, true);

   println!("[Server]  The ip of the server is {}:{}", local_ip().unwrap(), port);


   /* ===============================
          Init all server variable
      ==============================*/
   // Flag showing if the server is running or not
   let mut b_running : bool = false;

   // The soket used by the server to exchange datagrams with clients
   let socket = UdpSocket::bind(format!("{}:{}", "0.0.0.0", port.parse::<i16>().unwrap())).await.unwrap();
   let SocketRef = Arc::new(socket);

   // Address and port of the server
   let address =  String::new();
   let port: i16 = port.parse::<i16>().unwrap();

   // Root topic which can be subscribed by clients
   // Every topics have sub topic to them, you can go through each one like in a tree
   let root = Mutex::new(TopicV2::new(1,"/".to_string())); // default root topics is "/"
   let root_ref = Arc::new(root);

   // List of clients represented by their address and their ID
   let mut clients: Mutex<HashMap<u64, SocketAddr>> = Mutex::new(HashMap::default()); // <Client ID, address>
   let clientsRef = Arc::new(clients);

   // List of clients ping
   let clients_ping: HashMap<u64, u128> = HashMap::default(); // <Client ID, Ping in timestamp>
   let pings: Arc<Mutex<HashMap<u8, u128>>> = Arc::new(Mutex::new(HashMap::default()));

   println!("[Server] Server variables successfully initialized");

   // =============================
   //    Spawning async functions
   // =============================
   b_running = true;
   #[allow(unused)]
   tokio::spawn(datagrams_handler(SocketRef.clone(), clientsRef.clone(), root_ref.clone())).await;
   #[allow(unused)]
   tokio::spawn(ping_sender(SocketRef.clone(), clientsRef.clone(), )).await;

   println!("[Server] Server is running ...");
}

/*
   This method handle every incoming datagram in the broker
   @param receiver Arc<UdpSocket> : An atomic reference of the UDP socket of the server
 */
async fn datagrams_handler(
   receiver : Arc<UdpSocket>,
   clients_ref: Arc<Mutex<HashMap<u64, SocketAddr>>>,
   root_ref: Arc<Mutex<TopicV2>>
){
   println!("[Server Handler] Datagrams Handler spawned");
   // infinite loop receiving listening for datagrams
   loop {
      // 1 - create an empty buffer of size 1024
      let mut buf = [0; 1024];

      // 2 - Wait for bytes reception
      match receiver.recv_from(&mut buf).await {
         // 3 - Once bytes are received, check for errors
         Ok((n, src)) => {
            // 4 - if OK match on the first byte (MESSAGE_TYPE)
            println!("[Server Handler] Received {} bytes from {}", n, src);

            match MessageType::from(buf[0]) {
               MessageType::CONNECT => {
                  // 4.1 - A user is trying to connect to the server
                  println!("[Server Handler] {} is trying to connect", src.ip());
                  handle_connect(src, clients_ref.clone(), receiver.clone()).await;
               }
               MessageType::DATA => {
                  // 4.2 - A user is trying to sent data to the server
                  println!("[Server Handler] {} is trying to sent data", src.ip());
               }
               MessageType::OPEN_STREAM => {
                  // 4.3 - A user is trying to open a new stream
                  println!("[Server Handler] {} is trying to open a new stream", src.ip());
               }
               MessageType::SHUTDOWN => {
                  // 4.4 - A user is trying to shutdown the connexion with the server
                  println!("[Server Handler] {} is trying to shutdown the connexion with the server", src.ip());
               }
               MessageType::HEARTBEAT => {
                  // 4.5 - A user is trying to sent an heartbeat
                  println!("[Server Handler] {} is trying to sent an heartbeat", src.ip());
               }
               MessageType::OBJECT_REQUEST => {
                  // 4.6 - A user is trying to request an object
                  println!("[Server Handler] {} is trying to request an object", src.ip());
               }
               MessageType::TOPIC_REQUEST => {
                  // 4.7 - A user is trying to request a new topic
                  println!("[Server Handler] {} is trying to request a new topic", src.ip());
                  let topic_path = String::from_utf8(buf[1..].to_vec()).unwrap();

                  let topic_id = create_topics(&topic_path, root_ref.clone()).await;

                  let result = receiver.send_to(&RQ_TopicRequest_ACK::new(TopicsResponse::SUCCESS, topic_id).as_bytes(), src).await;
                  match result {
                     Ok(bytes) => {
                        println!("[Server Handler] Send {} bytes to {}", bytes, src.ip());
                     }
                     Err(_) => {
                        println!("[Server Handler] Failed to send ACK to {}", src.ip());
                     }
                  }
               }
               MessageType::PONG => {
                  // 4.8 - A user is trying to answer a ping request
                  println!("[Server Handler] {} is trying to answer a ping request", src.ip());
               }
               MessageType::TOPIC_REQUEST_ACK | MessageType::OBJECT_REQUEST_ACK | MessageType::CONNECT_ACK | MessageType::HEARTBEAT_REQUEST | MessageType::PING => {
                  // 4.1 - A user is trying to connect to the server
                  println!("[Server Handler] {} has sent an invalid datagram.", src.ip());
               }
               MessageType::UNKNOWN => {
                  println!("[Server Handler] Received unknown packet from {}", src.ip())
               }
            }

            //println!("{}", String::from_utf8_lossy(&buf[..n]));
         }
         Err(e) => {
            println!("Error: {}", e);
         }
      }
   }
}

/*
   This method handle every incoming datagram in the broker
   @param receiver Arc<UdpSocket> : An atomic reference of the UDP socket of the server
 */
async fn ping_sender(sender : Arc<UdpSocket>, clients : Arc<Mutex<HashMap<u64, SocketAddr>>>) {

}


