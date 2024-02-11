# coppervox logic library

This library handles running the logic circuits generated.
This includes:
- the behavior of each logic components
- the rendering of the components
- handling events interacting with the circuit


## Components

This library contains multiple bevy's components and systems for running logic circuits in realtime.
The main component is the `WireState` which needs to be set on all wires entities in the circuit.
It store the result of any other input component connected to it and provide a value for all output component touching it.

