Decouple the action log from the gamestate. This way laat_printed and so on
can be removed from the game state. Implement new ServerMsg variants for
when a player takes action. Do not treat bots differently, the
client does not need to know. 
This also makes it so that gamestate is smaller and cheaper to send. 
Refactor all call sites, do not add anything for backward compatibility, this
is a breaking change
