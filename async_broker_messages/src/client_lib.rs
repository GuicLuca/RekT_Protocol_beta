// This document contain all use-full method used by the client struct.
// @author : GuicLuca (lucasguichard127@gmail.com)
// date : 22/03/2023

use std::io::Error;
use tokio::sync::oneshot;
use bytes::Bytes;

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
    }
}
