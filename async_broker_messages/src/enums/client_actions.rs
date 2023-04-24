use crate::libs::types::{ClientSender, ObjectId, PingId, Responder, ServerSocket, TopicId};

/**
 * The ClientActions enum contain each action a user can do.
 * It will be used by the server task to ask the client struct
 * to execute something.
 */
#[derive(Debug)]
pub enum ClientActions {
    Get {
        key: String,
        resp: Responder<u128>,
    },
    HandleData {
        sender: ServerSocket,
        buffer: [u8; 1024],
    },
    AddSubscribedTopic{
        topic_ids: Vec<TopicId>
    },
    RemoveSubscribedTopic{
        topic_ids: Vec<TopicId>
    },
    AddSubscribedObject{
        object_id: ObjectId
    },
    RemoveSubscribedObject{
        object_id: ObjectId
    },
    StartManagers{
        server_sender: ServerSocket
    },
    UpdateServerLastRequest{
        time: u128
    },
    UpdateClientLastRequest{
        time: u128
    },
    HandlePong {
        ping_id: PingId, // The ping request that is answered
        current_time: u128, // Server time when the request has been received
    },
    HandleTopicRequest{
        server_socket: ServerSocket,
        buffer: [u8; 1024],
        client_sender: ClientSender
    },
    HandleDisconnect{
    },
    HandleObjectRequest{
        buffer: [u8; 1024],
        server_socket: ServerSocket,
        client_sender: ClientSender,
    }
}