use std::thread;

use regex::Regex;

mod ps_client;
mod ps_server;
mod ps_common;

fn main() {
    println!("Broker_test");

    let is_server = ps_common::get_cli_input("Input 0 for server and 1 for client : ", "wtf", Some(&vec!["0".to_string(), "1".into()]), None, false) == "0";

    if is_server {
        let port = ps_common::get_cli_input("Input port : ", "wtf", None, None, true);
        let listener = ps_server::listen(port);
        loop {
            for ret in listener.incoming() {
                match ret {
                    Ok(stream) => {
                        println!("New connection: {}", stream.peer_addr().unwrap());
                        thread::spawn(move || {
                            // connection succeeded
                            ps_server::handle_client(&stream)
                        });
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                        /* connection failed */
                    }
                }
            }
        }
    } else {
        let reg = Regex::new(r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$").unwrap();

        let addr = ps_common::get_cli_input("Input addr : ", "wtf", None, Some(&reg), false);
        let port = ps_common::get_cli_input("Input port : ", "wtf", None, None, true);

        let stream = ps_client::connect(&addr, &port);
        let ret = ps_common::send_bytes(&stream, b"coucou");
        if !ret {
            println!("byte sending failed");
        }
    }
}
