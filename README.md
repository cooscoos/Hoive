# Hoive
The boardgame [Hive](https://en.wikipedia.org/wiki/Hive_(game)), written in Rust.

Choose one of the following directories and do `cargo run`:
- **client**: play locally (couch co-op) or on a Hoive game web server, or;
- **server**: host a Hoive game web server.

The Hoive directory contains the game logic.

![snapshot of the app](/misc/gameplay.png "snapshot of the app")

## To do

- install on virtual machine and list dependencies in readme
- play test


### Base game

- Spiders are bugged and can move <3 distance! It's because they can backtrack and leave the hive. Solve this another day.

### Client


- Local client is working again. Tidy it up
- Tidy up chatsess and integrate its behaviour with pmoore, adding in pillbug
- Write tests to probe local client's use of BoardActions and Ask:: enums.
- Try get http server working as an option.

- Figure out how to turn (features on in submodules)[https://doc.rust-lang.org/cargo/reference/features.html] 
- write some tests
- "Check whether there are any games available on the server, if there are you have to join the empty one": can change later to play with friends based on uid, private flag in db

Consider:
- Bevy with (egui)[https://github.com/emilk/egui] or Fyrox.


### Server

- coming along nicely, need to work up the client side to handle websocket
- (websockets)[https://github.com/actix/examples/tree/99d0afde28d14a0b641ac52de821d79fa244d50a/websockets/echo] are the solution to having a more responsive UI (and a chat feature) -- have websocket to notify when turn complete (then check db), same for ctrl+c or quitting. Need to rewrite client in (awc)[https://docs.rs/awc/latest/awc/] rather than reqwest because reqwest doesn't have websocket features.
- have a flag / option for running an html server. Client advanced options.

- Complete writing tests for server/tests/api.rs --- need to figure out how to create sessions in tests.


<!-- 
Things I wrote that no longer seem to apply:

- beetle rendering on stringboard is weird
- does pillbug sumoing need a bee check for either party? - I don't think this can ever happen given the other constraints


#### Refs

 [good ref](https://fdeantoni.medium.com/rust-actix-diesel-sqlite-d67a1c3ef0e) [good ref 2](https://github.com/vascokk/fullstack-rust/tree/main/server/src) [half done, now finish]



### "House rules"
Then it might be "fun" to add new animals in a non-standard version of the game e.g.:

* a centipede that can remove any adjacent (non-flying) animal permanently from that game (but then also dies), maybe also has limited moveset - moves like ladybird but with only 2 moves. Mosquitos copying centipede must die if used like centipede.
* a housefly that can move anywhere (including into small gaps an ant can't reach) for one turn (and then must fly back - if it can't return to its original spot, it dies for that game or is returned to player hand). Maybe it doesn't need to die or return, maybe it can fly freely but never land adjacent to bees or maybe even spiders so that you need to defend bee / other peices with spider. Maybe both are cool, I dunno.
* maybe other people have made custom hive peices that we can implement, search later.
 -->
