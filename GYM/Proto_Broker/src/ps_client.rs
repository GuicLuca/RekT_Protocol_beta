use std::net::TcpStream;

pub fn connect(ip: &String, port: &String) -> TcpStream {
    let mut addr = ip.clone();
    addr.push_str(":");
    addr.push_str(port);
    let ret = TcpStream::connect(addr.clone());
    match ret {
        Ok(stream) => {
            println!("connected to {addr}");
            return stream;
        }
        Err(e) => {
            println!("Error connecting to {addr}");
            println!("{}", e);
            panic!()
        }
    }
}