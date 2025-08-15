Some types are horribly long. Do they need to be that long? Can they be
simplified? Should they be simplified or is that already the best solution?

Add an ID to the request struct for player actions. That ID stands for the ID of
the player which should take the action. Make the bots playing automatically
optional. If they are not playing by themselves they can be controlled through
the same signals as the player. Bot auto playing should be a CLI parameter, but
also a parameter in the reset action, which already allows setting the bot
count. 


