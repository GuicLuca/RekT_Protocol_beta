// This document contain all alias type used in this project
// @author : GuicLuca (lucasguichard127@gmail.com)
// date : 01/04/2023

use std::collections::HashMap;
use std::io::Error;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;
use tokio::sync::{Mutex, oneshot, RwLock};
use crate::client_lib::ClientActions;



// ===================
//  Common used types
// ===================
pub type Size = u16;
pub type TopicId = u64;

// ===================
//    Server types
// ===================
pub type ServerSocket = Arc<UdpSocket>;
pub type ClientsHashMap<T> = Arc<RwLock<HashMap<ClientId, T>>>;
pub type TopicsHashMap<T> = Arc<RwLock<HashMap<TopicId, T>>>;
pub type PingsHashMap = Arc<Mutex<HashMap<u8, u128>>>;


// ===================
//   Clients types
// ===================
pub type ClientId = u64;
pub type Responder<T> = oneshot::Sender<Result<T, Error>>;
pub type ClientSender = Sender<ClientActions>;