use tokio::{
   io::{AsyncBufReadExt,AsyncBufRead, AsyncWriteExt, BufReader, ReadHalf, WriteHalf},
   net::{UdpSocket},
};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use local_ip_address::local_ip;

use crate::topic_v2::TopicV2;

mod ps_datagram_structs;
mod topic_v2;
mod ps_common;
mod ps_server_lib;
use crate::ps_datagram_structs::{MessageType, RQ_Connect_ACK_OK};
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
   let address =  String::new();
   let port: i16 = port.parse::<i16>().unwrap();
   let topics: HashMap<u64, Vec<u64>> = HashMap::default();
   let reader: ReadHalf<tokio::net::UdpSocket>;
   let writer: WriteHalf<tokio::net::UdpSocket>;
   let mut clients: HashMap<u64, SocketAddr> = HashMap::default();
   let root: TopicV2;
   let b_running : bool = false;
   let clients_ping: HashMap<u64, SocketAddr> = HashMap::default();
   let pings: HashMap<u8, u128> = HashMap::default();


   loop {
      let mut buf = [0; 1024];
      match socket.recv_from(&mut buf).await {
         Ok((n, src)) => {
            println!("Received {} bytes from {}", n, src);
            match MessageType::from(buf[0]) {
               MessageType::CONNECT => {
                  /* ====================
                     Handle new client
                   =====================*/
                  tokio::spawn(async move {
                     let (is_connected, current_id) = already_connected(&src.ip(), clients.clone());
                     let uuid;
                     let result;
                     if is_connected {
                        uuid = current_id;
                     } else {
                        uuid = get_new_id(clients.clone());
                        clients.insert(uuid, src);
                     }
                     result = socket.send_to(&RQ_Connect_ACK_OK::new(uuid, 1).as_bytes(), src).await;
                     match result {
                        Ok(bytes) => {
                           println!("Send {} bytes", bytes);
                        }
                        Err(_) => {
                           println!("Failed to send Connect ACK to {}", src);
                        }
                     }
                  });
               }
               MessageType::DATA => {}
               MessageType::OPEN_STREAM => {}
               MessageType::SHUTDOWN => {}
               MessageType::HEARTBEAT => {}
               MessageType::OBJECT_REQUEST => {}
               MessageType::TOPIC_REQUEST => {
                  /*    let topic_path = String::from_utf8(buf[1..].to_vec()).unwrap();

                      let topic_id = self.create_topics(&topic_path);

                      let result = self.socket.send_to(&RQ_TopicRequest_ACK::new(TopicsResponse::SUCCESS, topic_id).as_bytes(), src);
                      match result {
                          Ok(bytes) => {
                              println!("Send {} bytes", bytes);
                          }
                          Err(_) => {
                              println!("Failed to send ACK to {}", src);
                          }
                      }
                  */}
               MessageType::PONG => {}
               MessageType::PING => {/*Drop paquet*/}
               MessageType::TOPIC_REQUEST_ACK | MessageType::OBJECT_REQUEST_ACK | MessageType::CONNECT_ACK | MessageType::HEARTBEAT_REQUEST | MessageType::PONG => {
                  //invalid_msg_type(&src)
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


