# Hoive
The boardgame [Hive](https://en.wikipedia.org/wiki/Hive_(game)), written in Rust.

Choose one of the following directories and do `cargo run`:
- **client**: play locally (couch co-op) or on a Hoive game web server, or;
- **server**: host a Hoive game web server using websocket (default) or http.

The hoive directory contains the game logic.

![snapshot of the app](/misc/gameplay.png "snapshot of the app")

## To do

- install on virtual machine and list dependencies in readme
- play test

### Hoive base game
- Spiral decoder is broken because always assumes hex (0,0) has something in it. Need to be able to support an empty hex (0,0). Make spiral_decoding_no_origin test pass
- Spiders are bugged (hehe) and can move <3 distance It's because they can backtrack and leave the hive. Check hive connection at each step. Same for ladybird but with below.


### Client

- Write tests to probe local client's use of BoardActions and Req:: enums.

### Server

- Reinstate connection to db with pool
- Test that db games / users are being wiped on leave.


### Low priority

- Try get http server working as an option. - have a flag / option for running an html server. Client advanced options.
- Consider: Bevy, maybe with (egui)[https://github.com/emilk/egui] or Fyrox.
- display you: for chat
- fix colour of other players to random?
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
