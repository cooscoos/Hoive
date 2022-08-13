# Hoive
The boardgame Hive, written in Rust.

Done so far:
* play the game in terminal with cargo run
* basic rules working
* ants (a), spiders (s), ladybirds (l), queen bees (q), pillbugs (p), beetles (b)

## To do

### Rule tweaks

* Sort pillbug beetlegate. It's a special move so we just need to check for beetle gates one layer up using the source,dest intersection rule similar to beetle_check
* can ladybird move over beetle on top of other chip (yes) - can it during tests?


### Other animals
To finish:

* grashopper - line drawing - then a check to see if it's straight and only goes over occupied hexes
* mosquito - special move to become neighbour. Must be beetle if in layer >0.

### Online
Make it so that two people can play on t'internet and other weirdos can just watch.

### "House rules"
Then it might be fun to add new animals in a non-standard version of the game e.g.:

* a centipede that can remove any adjacent (non-flying) animal permanently from that game (but then also dies), maybe also has limited moveset - moves like ladybird but with only 2 moves. Mosquitos copying centipede must die if used like centipede.
* a housefly that can move anywhere (including into small gaps an ant can't reach) for one turn (and then must fly back - if it can't return to its original spot, it dies for that game or is returned to player hand). Maybe it doesn't need to die or return, maybe it can fly freely but never land adjacent to bees or maybe even spiders so that you need to defend bee / other peices with spider. Maybe both are cool, I dunno.
* maybe other people have made custom hive peices that we can implement, search later.

