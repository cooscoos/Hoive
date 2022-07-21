# Hoive
The boardgame Hive, written in Rust.

## To do


### Terminal interface

Finish coding terminal interface so that we can play simple game and start playtesting a bit.

* I think there's a bug with placement adjacency resulting from doubleheight to cube co-ords conversion
* Need to allow moves of existing tiles on board

### Tidy up

#### morphops.rs, board.rs, game.rs

* Functions are using a variety of vectors, HashSets, HashMaps, BTrees and sorting. Can we use and stick to one if converting between these is inefficient? Need to read up on memory usage.


#### board.rs
* modularise, maybe branch the rules out to another module
* make methods compatible with non 3-coordinate systems



### Ant (and the ant squeeze rule)

We now have ant_close, a function that has logic to prevent the white ant from entering the pit shown in image
![ant squeeze](/reference/ant_squeeze.jpeg "ant squeeze")

This is about as complicated a situation as we'll ever see.

Now just need to make the board run this at the start of every turn and keep note of what to block off for ants.

* (Later) spider and bee similar, but with movement range, see https://www.redblobgames.com/grids/hexagons/

