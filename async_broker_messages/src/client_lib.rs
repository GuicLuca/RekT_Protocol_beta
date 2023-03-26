// This document contain all use-full method used by the client struct.
// @author : GuicLuca (lucasguichard127@gmail.com)
// date : 22/03/2023

use std::collections::HashMap;
use std::io::Error;
use std::sync::{Arc};

use tokio::sync::{Mutex, oneshot};

// ===================
//   Common used type
// ===================
type Responder<T> = oneshot::Sender<Result<T, Error>>;


/**
 * The ClientActions enum contain each action a user can do.
 * It will be used by the server task to ask the client struct
 * to execute something.
 */
#[derive(Debug)]
pub enum ClientActions {
    Get {
        key: String,
        resp: Responder<u64>,
    },
    UpdateServerLastRequest{
        // no parameter.
    },
    UpdateClientLastRequest{
        // no parameter.
    },
    HandlePong {
        ping_id: u8, // The ping request that is answered
        current_time: u128, // Server time when the request has been received
        pings_ref: Arc<Mutex<HashMap<u8, u128>>>, // contain all ping request sent by the server
    },
}
