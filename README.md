# Hoive
The boardgame [Hive](https://en.wikipedia.org/wiki/Hive_(game)), written in Rust.

Play the game in terminal with cargo run.


![snapshot of the app](/reference/gameplay.png "snapshot of the app")

## To do
### Bugcheck and tidy up

- There's a bug where beetle at 0-4 can move over q at 0,0 - beetles can move any number of spaces?! oops.
- play test
- add load game -- could match on MoveStatus::Load to quit out, restart and emulate moves
- does pillbug sumoing need a bee check for either party?

### Online

Make this a server application so that people can play on the internet. Steps:

#### Board to string
- Need a way to convert the board into a string that can be stored on an sqlite db. [this guy](https://www.youtube.com/watch?v=U4ogK0MIzqk) uses a good codifying method called forsyth-edwards notation (FEN), which could be parsed in [hex-spiral](https://crates.io/crates/hex-spiral) format starting at 0,0.
- Other attempts at [notation](https://github.com/mtorpey/hive-notation) are trying to be clever with fixed board orientation to be human-readable. This makes them more verbose. 
- If we went for something similar to FEN but with a spiral we'd have something good enough to store a board's state succinctly. Could even be used for saving games without having to re-emulate them (although re-emulating is great for testing).

capital letter = black peices, non-cap = white
Qq[6]A1a2(b1(B2))a3[1]

This means black queen at 0,0, then white queen at 0,-2, then 6 gaps, the black ant 1, then ant 2, with a black beetle above it, with a black beetle 2 above that... etc

- Write a function which can save and load these strings

#### Sqlite db

- Then create a db using diesel to store game state, then interact with the game via this db [good ref](https://fdeantoni.medium.com/rust-actix-diesel-sqlite-d67a1c3ef0e) [good ref 2](https://github.com/vascokk/fullstack-rust/tree/main/server/src) [half done, now finish]



#### Create API
- Now modularise pmoore and make it more like a general api that plays with the db can be interacted with by any front-end. To start, the front end will still be the (local) terminal window

#### Remote access
- Now make it so you can use a remote terminal, [this](https://github.com/vascokk/fullstack-rust/tree/main/server/src) is a good reference
- You then have a server. Could later create a web-based front end.
<!-- 
### "House rules"
Then it might be "fun" to add new animals in a non-standard version of the game e.g.:

* a centipede that can remove any adjacent (non-flying) animal permanently from that game (but then also dies), maybe also has limited moveset - moves like ladybird but with only 2 moves. Mosquitos copying centipede must die if used like centipede.
* a housefly that can move anywhere (including into small gaps an ant can't reach) for one turn (and then must fly back - if it can't return to its original spot, it dies for that game or is returned to player hand). Maybe it doesn't need to die or return, maybe it can fly freely but never land adjacent to bees or maybe even spiders so that you need to defend bee / other peices with spider. Maybe both are cool, I dunno.
* maybe other people have made custom hive peices that we can implement, search later.
 -->
