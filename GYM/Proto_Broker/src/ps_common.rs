use std::io::{BufRead, Write};
use std::net::TcpStream;

pub fn send_bytes(mut stream : &TcpStream, bytes : &[u8]) -> bool {
    let size = stream.write(bytes);
    if size.is_err() {
        println!("{}", size.unwrap_err());
        return false;
    }

    let percentage = (size.unwrap() / bytes.len()) * 100;
    println!("{percentage}% bytes sent");
    return true;
}

pub fn receive_bytes(mut stream : &TcpStream, mut bytes : &mut Vec<u8>) {
    let mut reader = std::io::BufReader::new(&mut stream);

    // Read current current data in the TcpStream
    bytes.write(reader.fill_buf().unwrap()).expect("how"); 
    reader.consume(bytes.len());
}