use std::io::{BufRead, Write};
use std::net::TcpStream;

pub fn send_bytes(mut stream : &TcpStream, bytes : &[u8]) -> bool {
    let size = stream.write(bytes);

    if size.is_err() {
        println!("{}", size.unwrap_err());
        return false;
    }

    let percentage = size.unwrap() / bytes.len();
    println!("{percentage}% bytes sent");
    return true;
}

pub fn receive_bytes(mut stream : &TcpStream, mut bytes : &Vec<u8>) -> Vec<u8> {
    let mut reader = std::io::BufReader::new(&mut stream);

    // Read current current data in the TcpStream
    let received: Vec<u8> = reader.fill_buf().unwrap().to_vec();
    reader.consume(received.len());
    return received;
}