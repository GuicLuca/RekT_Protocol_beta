use std::io::{stdin, stdout, Write};
use std::thread;
use regex::Regex;

mod ps_client;
mod ps_server;
mod ps_common;

fn main() {
    println!("Broker_test");

    let mut is_valid = false;
    let mut value: String = String::new();

    while !is_valid {
        value = String::from("");
        println!("Input 0 for server and 1 for client : ");
        stdin().read_line(&mut value).expect("C cassé");
        value = value.trim().parse().unwrap();
        if value == "0" || value == "1" {
            is_valid = true;
        } else {
            println!("wtf");
        }
    }

    let is_server = value == "0";
    let mut is_valid = false;

    if is_server {
        while !is_valid {
            value = String::from("");
            println!("Input port : ");
            stdin().read_line(&mut value).expect("??");
            value = value.trim().parse().unwrap();
            if !value.parse::<i16>().is_err() {
                is_valid = true;
            } else {
                println!("wtf")
            }
        }
        let listener = ps_server::listen(value);
        loop {
            for ret in listener.incoming(){
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
        while !is_valid {
            value = String::from("");
            println!("Input addr : ");
            stdin().read_line(&mut value).expect("??");
            value = value.trim().parse().unwrap();
            if reg.is_match(&value) {
                is_valid = true;
            } else {
                println!("wtf")
            }
        }
        let addr = value.clone();
        is_valid = false;
        while !is_valid {
            value = String::from("");
            println!("Input port : ");
            stdin().read_line(&mut value).expect("??");
            value = value.trim().parse().unwrap();
            if !value.parse::<i16>().is_err() {
                is_valid = true;
            } else {
                println!("wtf")
            }
        }
        let port = value.clone();
        let stream = ps_client::connect(&addr, &port);
        let ret = ps_common::send_bytes(&stream, b"coucou");
        if !ret{
            println!("byte sending failed");
        }
    }
}
