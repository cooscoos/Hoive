# Hoive
The boardgame [Hive](https://en.wikipedia.org/wiki/Hive_(game)), written in Rust.

Choose one of the following directories and do `cargo run`:
- **client**: play game locally (couch co-op), or on a websocket server.
- **server**: host a websocket server.

The hoive directory contains the game logic.

![snapshot of the app](/misc/gameplay.png "snapshot of the app")

## To do

- Server: reinstate connection to db with pool
- Install on virtual machine to check and list dependencies
- Play test to spot bugs in base game


### Low priority

- Option to select an empty game to join based on its id in websocket version so can play with friends on server


### Useful references
- [websockets](https://github.com/actix/examples/tree/99d0afde28d14a0b641ac52de821d79fa244d50a/websockets/echo)
- [http server games](https://github.com/vascokk/fullstack-rust/tree/main/server/src)
- [actix and diesel](https://fdeantoni.medium.com/rust-actix-diesel-sqlite-d67a1c3ef0e)


<!-- 
Things I wrote that no longer seem to apply:

- beetle rendering on stringboard is weird
- does pillbug sumoing need a bee check for either party? - I don't think this can ever happen given the other constraints
- Figure out how to turn (features on in submodules)[https://doc.rust-lang.org/cargo/reference/features.html] 

### "House rules"
Then it might be "fun" to add new animals in a non-standard version of the game e.g.:

* a centipede that can remove any adjacent (non-flying) animal permanently from that game (but then also dies), maybe also has limited moveset - moves like ladybird but with only 2 moves. Mosquitos copying centipede must die if used like centipede.
* a housefly that can move anywhere (including into small gaps an ant can't reach) for one turn (and then must fly back - if it can't return to its original spot, it dies for that game or is returned to player hand). Maybe it doesn't need to die or return, maybe it can fly freely but never land adjacent to bees or maybe even spiders so that you need to defend bee / other peices with spider. Maybe both are cool, I dunno.
* maybe other people have made custom hive peices that we can implement, search later.
 -->
