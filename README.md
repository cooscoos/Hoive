# Hoive

The boardgame Hive, written in Rust.

Done so far:
* play the game in terminal with cargo run
* basic rules working
* ants (a), spiders (s), ladybirds (l) and queen bees (q)

![snapshot](/reference/snapshot.png "snapshot of game")


## To do

### Pillbug

Seems to be working:

* write a module to replay a game based on what I do, need to have a savefn for pmoore and loadup in tests
* write tests for those
* then write tests for pillbug and various error messages including hive splits etc utilising your new module
* tidy up a lot -- pmoore, board and specials and their interactions need to be refactored



### Other animals

This order seems sensible: 

* pillbug - forcing other player's move but with limited checks (still need hive breakcheck), plus update pmoore for sumo list and sumo action
* beetle
* then back to pillbug to capture the beetle gate rule
* grashopper - line drawing - then a check to see if it's straight and only goes over occupied hexes
* mosquito


### "House rules"

Then it might be fun to add new animals in a non-standard version of the game e.g.:

* a centipede that can remove any adjacent (non-flying) animal permanently from that game (but then also dies), maybe also has limited moveset - moves like ladybird but with only 2 moves. Mosquitos copying centipede must die if used like centipede.
* a housefly that can move anywhere (including into small gaps an ant can't reach) for one turn (and then must fly back - if it can't return to its original spot, it dies for that game or is returned to player hand). Maybe it doesn't need to die or return, maybe it can fly freely but never land adjacent to bees or maybe even spiders so that you need to defend bee / other peices with spider. Maybe both are cool, I dunno.
* maybe other people have made custom hive peices that we can implement, search later.

### Save and load in txt file

Using the ascii text parsers to save games and load them up, or view them playing out step by step, might be quite useful for tests and demoing strategies. Low priority really.

### Online

Make it so that two people can play on t'internet and other weirdos can just watch.

### Tidy up

Always.

#### morphops.rs, board.rs, game.rs

Functions are using a variety of vectors, HashSets, HashMaps, BTrees and sorting. Can we use and stick to one if converting between these is inefficient? Need to read up on memory usage.

#### board.rs
* modularise, maybe branch the rules out to another module
* make methods compatible with non 3-coordinate systems

#### tests

* Scale back the number of new tests we need to write for new co-ord systems. Writing a test that does a conversion from X co-ord to cube successfully should do a lot to show that the game can run in that new co-ordinate system.