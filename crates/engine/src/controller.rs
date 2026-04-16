/*
The purpose of this file is to define the control loop around the interpreter.
The interpreter allows us to execute a given step given its input buffer,
and returns whether or not that step was successful (StepResult::Ok), if it required input (StepResult::NeedsInput),
if the game is over (StepResult::GameOver), or if there was an error (StepResult::Error).

The job of the controller is to call interpreter::step() until input is required, the game is over or there is an error.
In each of these cases, the controller forwards the step result to the Player Interface.
If the controller gets input back from the player interface, it feeds that input into the input buffer of the interpreter and the process repeats.

*/
