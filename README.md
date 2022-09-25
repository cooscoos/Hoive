# Hoive
The boardgame [Hive](https://en.wikipedia.org/wiki/Hive_(game)), written in Rust.

Play the game in terminal with cargo run.


![snapshot of the app](/reference/gameplay.png "snapshot of the app")

## To do
### Base game

- There's a bug where beetle at 0-4 can move over q at 0,0 - beetles can move any number of spaces?! oops.
- beetle rendering on stringboard is weird
- play test
- does pillbug sumoing need a bee check for either party?


### Client


- everything broadly works. have a good old tidy up and write some tests


- try remove traits like deserialize and see if you can get away with it
- finish act in ui to make movement request instead of updating board
- oberve fn: A better UI would get usr input to feel responsive ... but also poll for update while waiting. Might need some other fn polling in the background with tx,rx to achieve that
- observe fn also ques user garbage typing into next move - this is not good


A front end for Hoive
- Solution to quitting whenever might be tx,rx (something for later)
- "Check whether there are any games available on the server, if there are you have to join the empty one": can change later to play with friends based on uid, private flag in db

Consider:
- Bevy with (egui)[https://github.com/emilk/egui]


### Server

Make this a server application so that people can play on the internet. Steps:

- everything broadly works --have a good old tidy up
- server will need to have some sort of history for tracking pillbugs, use gamestate

#### Refs

- Create a db using diesel to store and load game state, then interact with the game via this db [good ref](https://fdeantoni.medium.com/rust-actix-diesel-sqlite-d67a1c3ef0e) [good ref 2](https://github.com/vascokk/fullstack-rust/tree/main/server/src) [half done, now finish]


<!-- 
### "House rules"
Then it might be "fun" to add new animals in a non-standard version of the game e.g.:

* a centipede that can remove any adjacent (non-flying) animal permanently from that game (but then also dies), maybe also has limited moveset - moves like ladybird but with only 2 moves. Mosquitos copying centipede must die if used like centipede.
* a housefly that can move anywhere (including into small gaps an ant can't reach) for one turn (and then must fly back - if it can't return to its original spot, it dies for that game or is returned to player hand). Maybe it doesn't need to die or return, maybe it can fly freely but never land adjacent to bees or maybe even spiders so that you need to defend bee / other peices with spider. Maybe both are cool, I dunno.
* maybe other people have made custom hive peices that we can implement, search later.
 -->
