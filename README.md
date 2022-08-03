# Hoive

The boardgame Hive, written in Rust.

Done so far:
* play the game in terminal with cargo run
* basic rules working
* ants (a), spiders (s), ladybirds (l), queen bees (q), pillbugs (p)

## To do

### Figure out how to benchmark

* How do we benchmark
* then improve double for loop with elem/elem2 can be solved w/ BTree, see https://www.reddit.com/r/rust/comments/wdb1uo/hey_rustaceans_got_a_question_ask_here_312022/iimrdim/?context=3
* read this: https://doc.rust-lang.org/std/collections/index.html
### Tidy up

* tidy up a lot -- pmoore, board and specials and their interactions need to be refactored


### Other animals

This order seems sensible: 

* beetle (dread)
* back to pillbug to capture the beetle gate rule
* grashopper - line drawing - then a check to see if it's straight and only goes over occupied hexes
* mosquito


### "House rules"

Then it might be fun to add new animals in a non-standard version of the game e.g.:

* a centipede that can remove any adjacent (non-flying) animal permanently from that game (but then also dies), maybe also has limited moveset - moves like ladybird but with only 2 moves. Mosquitos copying centipede must die if used like centipede.
* a housefly that can move anywhere (including into small gaps an ant can't reach) for one turn (and then must fly back - if it can't return to its original spot, it dies for that game or is returned to player hand). Maybe it doesn't need to die or return, maybe it can fly freely but never land adjacent to bees or maybe even spiders so that you need to defend bee / other peices with spider. Maybe both are cool, I dunno.
* maybe other people have made custom hive peices that we can implement, search later.

### Online

Make it so that two people can play on t'internet and other weirdos can just watch.

### Tweaks

#### morphops.rs, board.rs, game.rs

Functions are using a variety of vectors, HashSets, HashMaps, BTrees and sorting. Can we use and stick to one if converting between these is inefficient? Need to read up on memory usage.

#### board.rs
* modularise, maybe branch the rules out to another module
* make methods compatible with non 3-coordinate systems

#### tests

* Scale back the number of new tests we need to write for new co-ord systems. Writing a test that does a conversion from X co-ord to cube successfully should do a lot to show that the game can run in that new co-ordinate system.