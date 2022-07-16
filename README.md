# Hoive
The boardgame Hive, written in Rust.

## To do

### Ant (and the ant squeeze rule)

Need a rule that prevents white ant from entering the pit shown in image
![ant squeeze](/reference/ant_squeeze.jpeg "ant squeeze")

This is about as complicated a situation as we'll ever see.

Thoughts:

* to prevent squeeze, could bridge short gaps with ghost hexes, and then fill voids with ghost hexes
* ghost hexes affect ant/spider/bee only
* ant can then move based on existing constraints
* this would avoid needing to develop "path planning with obstacles" or move-by-move checks on the ant (could be inefficient). The ant just needs ghost tile guides before it moves.
* (Later) spider and bee similar, but with movement range, see https://www.redblobgames.com/grids/hexagons/
