extern crate core;

use std::thread::sleep;
use std::time::Duration;

use regex::Regex;

use crate::ps_udp_client::Client;
use crate::ps_udp_server::Server;

mod ps_common;
mod ps_udp_server;
mod ps_udp_client;
mod ps_client;
mod ps_datagram_structs;
mod topic;

fn main() {
    println!("Broker_test");

    let is_server = ps_common::get_cli_input("Input 0 for server and 1 for client : ", "wtf", Some(&vec!["0".to_string(), "1".into()]), None, false) == "0";

    if is_server {
        let port = ps_common::get_cli_input("Input port : ", "wtf", None, None, true);

        let mut serv = Server::serve("0.0.0.0".to_string(), port.parse::<i16>().unwrap());
        serv.main_loop()
    } else {
        let reg = Regex::new(r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$").unwrap();

        println!("0 : Manual Mode");
        println!("1 : Mat PC C8 | 143");
        println!("2 : Hugo PC C8 | 109");
        println!("3 : Lucas PC C8 | 125");
        println!("4 : Taylor Swift C8 | 129");
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
                addr = String::from("192.168.0.126")
            }
            _ => {
                println!("fallback to manual mode");
                addr = ps_common::get_cli_input("Input addr : ", "wtf", None, Some(&reg), false);
            }
        }

        let port = ps_common::get_cli_input("Input port : ", "wtf", None, None, true);

        let full_addr = format!("{}:{}", addr.trim(), port.trim());


        let client = Client::connect(full_addr.clone());
        println!("your id is : {}",client.get_id());
    }
}
