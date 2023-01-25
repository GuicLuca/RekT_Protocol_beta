use std::io::prelude::*;
use std::net::TcpStream;
use std::io;
use std::io::{stdin, stdout};

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("192.168.0.108:3838").expect("Connection failed to listener.");

    while true {
        let mut input:String = String::new();
        stdin().read_line(&mut input);
        if(input == "quit"){
            break;
        }
        let bytes_written = stream.write(input.as_bytes()).unwrap();
        if bytes_written < input.len() {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                format!("Sent {}/{} bytes", bytes_written, input.len()),
            ));
        }
        stream.flush();
    };


    Ok(())
} // the stream is closed here