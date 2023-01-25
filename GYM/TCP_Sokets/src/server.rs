use std::io::{Read, Write,BufRead};
use std::net::{TcpListener, TcpStream};
use std::thread;

//Server that can read string bytes

fn handle_client(mut stream: TcpStream) {
    loop {
        let mut reader = std::io::BufReader::new(&mut stream);

        // Read current current data in the TcpStream
        let received: Vec<u8> = reader.fill_buf().unwrap().to_vec();
        reader.consume(received.len());
        let my_string = String::from_utf8(received).unwrap();

        println!("{my_string}");
    }

}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:3838").unwrap();
    println!("Server running on port 3838");

    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move || {
                    // connection succeeded
                    handle_client(stream)
                });
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }

    // close the socket server
    drop(listener);
}