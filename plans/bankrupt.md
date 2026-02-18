Player/Bot bancrupty is not handled well at the moment. If I have 0 chips, I
cannot bet anything. Handling this is also pretty complicated which is why right
now it is not done. 

I propose a simple way to handle this:

If any player goes bankrupt, (not when they go all in), but when after a round a
player has 0 chips, then come up with a new state variant: not playable

The CLI and WASM clients should show a message saying that all players need to
have chips to continue playing and that an entirely new game should be started. 

On the server, wait for a new game message, and if a non-playable state is
present, respond to messages which are at the moment unsupported with a fitting
error.

This might also simplify some checks, look if that is the case. 
