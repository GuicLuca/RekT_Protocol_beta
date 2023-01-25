use std::io::prelude::*;
use std::net::TcpStream;
use std::io;
use std::io::{stdin, stdout};


/**
 * Thread that listent to message from server
 */
fn handle_server_message(mut stream: TcpStream) {
    loop {
        let mut reader = std::io::BufReader::new(&mut stream);

        // Read current current data in the TcpStream
        let received: Vec<u8> = reader.fill_buf().unwrap().to_vec();
        reader.consume(received.len());
        let source_message = String::new(received.pop());
        let message = String::from_utf8(received).unwrap();

        println!("[{my_string}] : {message}");
    }

}

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