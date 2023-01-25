# TCP server-clients system

1. check the server ip with ``ipconfig /all`` 
2. replace the server ip in `/src/client.rs`
3. Start one server and many clients
4. try to send messages with anyone and ensure that every peer send/receive all messages.

### connection :
When a client connect to the server, the serve open a new stream and handle incoming message in a new thread.
This new stream is saved in the vector "opened_stream". On the client, a new stream is opened to handle incoming message from the server.

### data exchange :
#### Client :
In the main function a loop wait for string inputs and send it to the server.
#### Server :
When the server receive something from any client, it sent it to all opened stream (except the sender) after adding a header.
The header contain the sender ID managed on the server. So the client can print "\[sender id] : this is my message bla bla bla".
 