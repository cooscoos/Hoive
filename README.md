# Hoive
The boardgame Hive, written in Rust.

Play the game in terminal with cargo run.


## To do
### Mosquito

- done, tidy up pmoore etc and then have a good playtest and write some tests because that was a nighmare.

### Tidy up

- does sumo need a bee check?
- Have a good tidy up and benchmark of the code once all animals done, and start to think about how it interacts with online module.

### Online
Make it so that two people can play on t'internet and other weirdos can just watch.

### "House rules"
Then it might be fun to add new animals in a non-standard version of the game e.g.:

* a centipede that can remove any adjacent (non-flying) animal permanently from that game (but then also dies), maybe also has limited moveset - moves like ladybird but with only 2 moves. Mosquitos copying centipede must die if used like centipede.
* a housefly that can move anywhere (including into small gaps an ant can't reach) for one turn (and then must fly back - if it can't return to its original spot, it dies for that game or is returned to player hand). Maybe it doesn't need to die or return, maybe it can fly freely but never land adjacent to bees or maybe even spiders so that you need to defend bee / other peices with spider. Maybe both are cool, I dunno.
* maybe other people have made custom hive peices that we can implement, search later.

