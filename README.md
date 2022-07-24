# Hoive
The boardgame Hive, written in Rust.

## To do

### Tests for bee and win state

All coded up. Write tests for the below:
* Have to place bee by turn 5, prohibit player movement of existing chips before bee is placed
* Win state if opponent's bee has 6 neighbours at the end of your turn
* Win state for opponent if your bee has 6 neighbours (sepuku)


### Other animals

This order seems sensible: 

* ladybird
* beetle
* grashopper
* pillbug
* mosquito

### Tidy up

Always. Once everything is working I want to make this as efficient as it can be.

#### Non-existent hex in doubleheight

Prevent player from choosing hexes which don't exist in pmoore (e.g. 0,-5). Could make megabugs.

#### morphops.rs, board.rs, game.rs

Functions are using a variety of vectors, HashSets, HashMaps, BTrees and sorting. Can we use and stick to one if converting between these is inefficient? Need to read up on memory usage.

#### board.rs
* modularise, maybe branch the rules out to another module
* make methods compatible with non 3-coordinate systems

#### tests

* Scale back the number of new tests we need to write for new co-ord systems. Writing a test that does a conversion from X co-ord to cube successfully should do a lot to show that the game can run in that new co-ordinate system.

