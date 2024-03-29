extern crate core;

use regex::Regex;
use crate::ps_udp_client::Client;
mod ps_common;
mod ps_udp_client;
mod ps_client;
mod ps_datagram_structs;
mod topic;
mod topic_v2;

#[tokio::main]
async fn main() {
    println!("Hiii client !");
        let reg = Regex::new(r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$").unwrap();

        println!("0 : Manual Mode");
        println!("1 : Mat PC C8 | 143");
        println!("2 : Hugo PC C8 | 109");
        println!("3 : Lucas PC C8 | 125");
        println!("4 : Taylor Swift C8 | 129");
        println!("5 : Lucas PC Hotspot | 223");
        let choice: u8 = ps_common::get_cli_input("Choose server : ", "wtf", None, None, true).parse().unwrap();

        let addr;
        match choice {
            1 => {
                addr = String::from("192.168.0.143")
            }
            2 => {
                addr = String::from("192.168.0.109")
            }
            3 => {
                addr = String::from("192.168.0.125")
            }
            4 => {
                addr = String::from("192.168.0.129")
            }
            5 => {
                addr = String::from("172.19.132.246")
            }
            _ => {
                println!("fallback to manual mode");
                addr = ps_common::get_cli_input("Input addr : ", "wtf", None, Some(&reg), false);
            }
        }

        let port = ps_common::get_cli_input("Input port : ", "wtf", None, None, true);

        let full_addr = format!("{}:{}", addr.trim(), port.trim());


        let mut client = Client::connect(full_addr.clone()).await;
        println!("your id is : {}",client.get_id());
        println!("{}", client.create_topic_test().await.unwrap());
        client.wait_xd().await;
}
