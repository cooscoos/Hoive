# Hoive

The boardgame Hive, written in Rust.

Done so far:
* play the game in terminal with cargo run
* basic rules working
* ants, spiders, ladybirds and queen bees

![snapshot](/reference/snapshot.png "snapshot of game")


## To do

* create sumo handle in pmoore for pillbug selection (with selection of neighbours), then co-ord to sumo to
* make the sumo rules, ideally in animals.rs



### Other animals

This order seems sensible: 

* pillbug - forcing other player's move but with limited checks (still need hive breakcheck), plus update pmoore for sumo list and sumo action
* grashopper - line drawing - then a check to see if it's straight and only goes over occupied hexes
* beetle
* mosquito

### Tidy up

Always.

#### morphops.rs, board.rs, game.rs

Functions are using a variety of vectors, HashSets, HashMaps, BTrees and sorting. Can we use and stick to one if converting between these is inefficient? Need to read up on memory usage.

#### board.rs
* modularise, maybe branch the rules out to another module
* make methods compatible with non 3-coordinate systems

#### tests

* Scale back the number of new tests we need to write for new co-ord systems. Writing a test that does a conversion from X co-ord to cube successfully should do a lot to show that the game can run in that new co-ordinate system.

