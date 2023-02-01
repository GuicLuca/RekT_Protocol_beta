use regex::Regex;
use crate::ps_client::Client;

use crate::ps_server::Server;

mod ps_client;
mod ps_common;
mod ps_server;

fn main() {
    println!("Broker_test");

    let is_server = ps_common::get_cli_input("Input 0 for server and 1 for client : ", "wtf", Some(&vec!["0".to_string(), "1".into()]), None, false) == "0";

    if is_server {

        let port = ps_common::get_cli_input("Input port : ", "wtf", None, None, true);
        let mut serv = Server::new("0.0.0.0".to_string(), port.parse::<i16>().unwrap());

        serv.serve();
    } else {
        let reg = Regex::new(r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$").unwrap();

        let addr = ps_common::get_cli_input("Input addr : ", "wtf", None, Some(&reg), false);
        let port = ps_common::get_cli_input("Input port : ", "wtf", None, None, true);


        let mut client = Client::new();
        client.connect(&addr, &port);
        let ret = client.send_bytes(b"coucou");
        if !ret {
            println!("byte sending failed");
        }
    }
}
