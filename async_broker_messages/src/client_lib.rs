// This document contain all use-full method used by the client struct.
// @author : GuicLuca (lucasguichard127@gmail.com)
// date : 22/03/2023

use tokio::sync::oneshot;

// ===================
//   Common used type
// ===================
pub type Result<T> = std::result::Result<T,Box<dyn std::error::Error>>;
type Responder<T> = oneshot::Sender<Result<T>>;



/**
 * The ClientActions enum contain each action a user can do.
 * It will be used by the server task to ask the client struct
 * to execute something.
 */
#[derive(Debug)]
enum ClientActions {
    HandleRequest {
        buffer: String,
    },
    Set {
        key: String,
        val: Bytes,
    }
}
