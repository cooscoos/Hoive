# Hoive
The boardgame Hive, written in Rust.

## To do

### Spider and bee

* Write tests for spider and bee
* Prohibit player movement of existing chips before bee is placed
* Ensure bee is played on or before player's turn 3 (i.e. if turn N for first_player or N+ for not first_player, check if bee's position is None. If it is, then must play bee)
* Code a win state if opponent's bee has 6 neighbours at the end of your turn

### Other animals

This order seems sensible: 

* ladybird
* beetle
* grashopper
* pillbug
* mosquito

### Tidy up

Always. Once everything is working I want to make this as efficient as it can be.

#### morphops.rs, board.rs, game.rs

Functions are using a variety of vectors, HashSets, HashMaps, BTrees and sorting. Can we use and stick to one if converting between these is inefficient? Need to read up on memory usage.

#### board.rs
* modularise, maybe branch the rules out to another module
* make methods compatible with non 3-coordinate systems

#### tests

* Scale back the number of new tests we need to write for new co-ord systems. Writing a test that does a conversion from X co-ord to cube successfully should do a lot to show that the game can run in that new co-ordinate system.

