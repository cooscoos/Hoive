# Hoive
The boardgame [Hive](https://en.wikipedia.org/wiki/Hive_(game)), written in Rust.

Do cargo run in the terminal within the following directories:

- hoive: Run and play the game locally (couch co-op)
- server: Host a web server to allow people to play online via a client
- client: Play on an active web server

![snapshot of the app](/misc/gameplay.png "snapshot of the app")

## To do
### Base game (hoive)

- There's a bug where beetle at 0-4 can move over q at 0,0 - beetles can move any number of spaces?! oops.
- beetle rendering on stringboard is weird
- play test
- does pillbug sumoing need a bee check for either party?

### Client

- everything broadly works. but need a good old tidy up and write some tests
- try remove traits like deserialize and see if you can get away with it
- oberve fn: A better UI would get usr input to feel responsive ... but also poll for update while waiting. Might need some other fn polling in the background with tx,rx to achieve that
- observe fn also ques user garbage typing into next move - this is not good
- Solution to quitting whenever might be tx,rx (something for later)
- "Check whether there are any games available on the server, if there are you have to join the empty one": can change later to play with friends based on uid, private flag in db

Consider:
- Bevy with (egui)[https://github.com/emilk/egui]


### Server

- everything broadly works --have a good old tidy up
- server will need to have some sort of history for tracking pillbugs, use gamestate in db

<!-- 
#### Refs

 [good ref](https://fdeantoni.medium.com/rust-actix-diesel-sqlite-d67a1c3ef0e) [good ref 2](https://github.com/vascokk/fullstack-rust/tree/main/server/src) [half done, now finish]



### "House rules"
Then it might be "fun" to add new animals in a non-standard version of the game e.g.:

* a centipede that can remove any adjacent (non-flying) animal permanently from that game (but then also dies), maybe also has limited moveset - moves like ladybird but with only 2 moves. Mosquitos copying centipede must die if used like centipede.
* a housefly that can move anywhere (including into small gaps an ant can't reach) for one turn (and then must fly back - if it can't return to its original spot, it dies for that game or is returned to player hand). Maybe it doesn't need to die or return, maybe it can fly freely but never land adjacent to bees or maybe even spiders so that you need to defend bee / other peices with spider. Maybe both are cool, I dunno.
* maybe other people have made custom hive peices that we can implement, search later.
 -->
