// This document contain the client struct. It is where all client data
// are saved and should be computed.
// @author : GuicLuca (lucasguichard127@gmail.com)
// date : 22/03/2023

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::Receiver;
use crate::client_lib::ClientActions;
use crate::config::LogLevel::Error;

pub struct Client {
    // Identifiers
    pub id: u64,
    socket: SocketAddr,
    receiver: Receiver<ClientActions>,

    // Connection checker
    last_request: RwLock<u128>, // Timestamp of the last request received from this user
    missed_heartbeats: u8, // amount of heartbeats_request no received
    ping: u128, // client ping in ms

    // Topics
    topics: RwLock<Vec<u64>>, // Contain each subscribed topic id
    requests_counter: RwLock<HashMap<u64,u8>>, // Contain the id of the last request on a topic for scheduling
}

impl Client {
    /**
     * Only give the id and the socket of the client to create a new one
     * @param id: u64 The server identifier of the client
     * @param socket: SocketAddr The socket of the client (used to sent data through network)
     */
    pub fn new(id:u64, socket: SocketAddr, receiver: Receiver<ClientActions>) -> Client {
        Client {
            id,
            socket,
            receiver,
            last_request: RwLock::from(0),
            missed_heartbeats: 0,
            ping: 0,
            topics: RwLock::from(Vec::new()),
            requests_counter: RwLock::from(HashMap::new())
        }
    }

    /**
     * This method save the current time as the last
     */
    pub fn save_request_timestamp(&mut self){
        // Get the writ access to the value
        let mut last_request_mut = self.last_request.write().unwrap();
        // Update it at now
        *last_request_mut = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(); // Current time in ms
    }

    /**
     * This is the main function that receive all data of the client
     * and which return result or process logic operation on the client
     * during his connection time.
     */
    pub async fn manager(&mut self){
        // while client is alive ...
        loop {
            // Receive actions
            while let Some(cmd) = self.receiver.recv().await {
                match cmd {
                    // This command return the value of the requested key
                    ClientActions::Get { key, resp } => {
                        match key.as_str(){
                            // list of valid key under :
                            "id" =>{
                                let res = self.id;
                                // Ignore errors
                                let _ = resp.send(Ok(res));
                            }
                            _ => {
                                let _ = resp.send(Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Bad key requested : \"{}\"", key))));
                            }
                        }
                    }

                } // end matc
            }
        }


    }
}