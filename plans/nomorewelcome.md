The welcome message no longer really serves a purpose. The underlying transport
layers take care of acknowleding that a connection has been established.

There should be a 'subscribe' ClientMsg which the client can send to the server,
which then gets the client connection added to a list of connections which want
to receive broadcast messages whenever anything like a state update (player or
bot takes action, new hand, anything else interesting) happens. If the transport
the client is using is not bidirectional (right now just HTTP is like that),
then answer with an error message saying 'not supported'

The architecture should be very transport agnostic, and chaning and expanding
the message enums should be possible without individually having to adjust
transport layers. This means the transport layers can send ServerMsg to clients,
receive ClientMsg from clients (the handling of which returns a ServerMsg) which
then get sent to the client. Use serde for everything. If that means a
non-standard REST API then that is okay. (Look into how serde and REST APIs work
here). 

The plans in this document will require adjusting the CLI and WASM clients. This
is a breaking change, do not add anything for backwards compatibility. Update
callsites, change some architectural things if necessary


