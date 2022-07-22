# Hoive
The boardgame Hive, written in Rust.

## To do

### Bugs

This move should be legal on white player's turn but is getting reported as a hive split.
![legal move](/reference/bug.png "legal move bug")



Could be an issue with janky display of doubleheight in terminal vs the actual gameboard in the background.
* Write a test to emulate this move automatically =FAILED=.
* Write a test to output the cube co-ordinates to check they're correct =PASSED=
* Write a test to emulate this move in cube co-ords so that we know game logic is sound =FAILED=
* If not, manually work through the hive-split algo on paper -- are there issues with raster scan?
* Write a test to report doubleheight co-ordinates and cubic co-ordinates of the chips in this image to make sure this is okay
* Could it be that the code thinks the active_player is black team?

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


### Don't forget to

* Prohibit movement before bee is placed