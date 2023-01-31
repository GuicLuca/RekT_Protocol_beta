use std::net::{TcpListener, TcpStream};
use std::str::from_utf8;
use crate::ps_common::receive_bytes;

pub fn listen(port : String) -> TcpListener {
    let mut addr = String::new();
    addr.push_str("0.0.0.0");
    addr.push_str(":");
    addr.push_str(port.as_str());
    let ret = TcpListener::bind(addr.clone());
    match ret {
        Ok(listener) => {
            println!("Running on {addr}");
            return listener;
        }
        Err(e) => {
            println!("Error listening on {addr}");
            println!("{e}");
            panic!()
        }
    }
}

pub fn handle_client(stream : &TcpStream){
    let bytes:Vec<u8> = vec![];
    receive_bytes(&stream, &bytes);
    println!("{}", String::from_utf8(bytes).unwrap());
}