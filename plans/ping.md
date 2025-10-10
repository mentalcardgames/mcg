
add a Ping and Pong messages to the ClientMsg and ServerMsg enums. If a client
send a Ping message, the server should respond with a Pong message. The variants
should not contain any data. Add a ping subcommand to the CLI, and make the
server log it if a ping message is received. The wasm frontend should not use
the message for now. 
