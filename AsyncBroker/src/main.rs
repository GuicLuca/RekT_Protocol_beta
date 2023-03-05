use tokio::{
   io::{AsyncBufReadExt,AsyncBufRead, AsyncWriteExt, BufReader, ReadHalf, WriteHalf},
   net::{UdpSocket},
};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use local_ip_address::local_ip;
use tokio::sync::mpsc::Receiver;
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
   println!("Hi there ! Chose the port of for the server :");
   let port = ps_common::get_cli_input("Input port : ", "wtf", None, None, true);

   println!("The ip of the server is {}:{}", local_ip().unwrap(), port);


   /* ===============================
          Init all server variable
      ==============================*/

   let socket = UdpSocket::bind(format!("{}:{}", "0.0.0.0", port.parse::<i16>().unwrap())).await.unwrap();
   let SocketRef = Arc::new(socket);
   let address =  String::new();
   let port: i16 = port.parse::<i16>().unwrap();
   let topics: HashMap<u64, Vec<u64>> = HashMap::default();
   let mut clients: Mutex<HashMap<u64, SocketAddr>> = Mutex::new(HashMap::default());
   let clientsRef = Arc::new(clients);
   let root = Mutex::new(TopicV2::new(1,"/".to_string()));
   let root_ref = Arc::new(root);
   let b_running : bool = false;
   let clients_ping: HashMap<u64, SocketAddr> = HashMap::default();
   let pings: HashMap<u8, u128> = HashMap::default();

   // =============================
   //    Spawning async functions
   // =============================
   #[allow(unused)]
   tokio::spawn(datagrams_handler(SocketRef.clone(), clientsRef.clone(), root_ref.clone())).await;
   #[allow(unused)]
   tokio::spawn(ping_sender(SocketRef.clone())).await;
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
   // infinite loop receiving listening for datagrams
   loop {
      // 1 - create an empty buffer of size 1024
      let mut buf = [0; 1024];

      // 2 - Wait for bytes reception
      match receiver.recv_from(&mut buf).await {
         // 3 - Once bytes are received, check for errors
         Ok((n, src)) => {
            // 4 - if OK match on the first byte (MESSAGE_TYPE)
            println!("Received {} bytes from {}", n, src);

            match MessageType::from(buf[0]) {
               MessageType::CONNECT => {
                  // 4.1 - A user is trying to connect to the server
                  handle_connect(src, clients_ref.clone(), receiver.clone()).await;
               }
               MessageType::DATA => {}
               MessageType::OPEN_STREAM => {}
               MessageType::SHUTDOWN => {}
               MessageType::HEARTBEAT => {}
               MessageType::OBJECT_REQUEST => {}
               MessageType::TOPIC_REQUEST => {
                  let topic_path = String::from_utf8(buf[1..].to_vec()).unwrap();

                  let topic_id = create_topics(&topic_path, root_ref.clone()).await;

                  let result = datagrams_sender(receiver.clone(), &RQ_TopicRequest_ACK::new(TopicsResponse::SUCCESS, topic_id).as_bytes(), src).await;
                  match result {
                     Ok(bytes) => {
                        println!("Send {} bytes", bytes);
                     }
                     Err(_) => {
                        println!("Failed to send ACK to {}", src);
                     }
                  }
               }
               MessageType::PING => {}
               MessageType::TOPIC_REQUEST_ACK | MessageType::OBJECT_REQUEST_ACK | MessageType::CONNECT_ACK | MessageType::HEARTBEAT_REQUEST | MessageType::PONG => {
                  invalid_msg_type(&src).await;
               }
               MessageType::UNKNOWN => {
                  println!("recieved unknown packet from {}", src.ip())
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

async fn ping_sender(sender : Arc<UdpSocket>) {

}

async fn datagrams_sender(sender : Arc<UdpSocket>, buffer : &[u8], src: SocketAddr) -> std::io::Result<usize> {
   sender.send_to(buffer,src).await
}


