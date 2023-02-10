# Hoive
The boardgame [Hive](https://en.wikipedia.org/wiki/Hive_(game)), written in Rust.

Choose one of the following directories and do `cargo run`:
- **client**: play game locally (couch co-op), or on a websocket server.
- **server**: host a websocket server.

The hoive directory contains the game logic.

![snapshot of the app](/misc/gameplay.png "snapshot of the app")

## Code examples

If you're a developer, this repo contains examples of:

- running a multiplayer game on a websocket server;
- interacting with an sqlite database of game states;
- using hexagonal coordinate systems (cubic, doubleheight, spiral) and translating between them, and;
- morphological operations on hexagonal grids.


## My to do list

- Check and list dependencies in readme (e.g. lib sqlite)
- Play test to spot bugs in base game

### Lower priority

- Option to select an empty game to join based on its id so can play with friends on server
- Sqlite connections are done with pool, but is using appdata secure? Can pool be created somewhere else in the backend and grabbed by wsgamesessions?

### Useful references

- [hexagonal grids](https://www.redblobgames.com/grids/hexagons/)
- [websockets](https://github.com/actix/examples/tree/99d0afde28d14a0b641ac52de821d79fa244d50a/websockets/echo)
- [http server games](https://github.com/vascokk/fullstack-rust/tree/main/server/src)
- [actix and diesel for sqlite](https://fdeantoni.medium.com/rust-actix-diesel-sqlite-d67a1c3ef0e)


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
