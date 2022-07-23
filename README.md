# Hoive
The boardgame Hive, written in Rust.

## To do

### Spider and bee

* Code spider and bee movement ranges, see https://www.redblobgames.com/grids/hexagons/
* Prohibit player movement of existing chips before bee is placed
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

