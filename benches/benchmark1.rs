// cargo benchmark
// This no longer works because of a change to coordinae systems.
// We compared 5 methods of finding mutual elements in two collections
// Conclusion - use HashSet with intersect
// use criterion::{criterion_group, criterion_main, Criterion};

// use hoive::game::{board::Board, history};
// use hoive::maths::coord::{Coord, Cube};

// use std::collections::{BTreeSet, HashSet};

// fn setup_board() -> Board<Cube> {
//     // Create and emulate a board from a named reference/tests/snapshots file
//     let mut board = Board::new(Cube);
//     history::emulate(&mut board, "snapshot_5neighbours".to_string(), true);
//     board
// }

// // Get 6 neighbouring tile co-ordinates in cube co-ordinates as a list
// fn neighbour_tiles(position: (i8, i8, i8)) -> [(i8, i8, i8); 6] {
//     let (q, r, s) = position;

//     [
//         (q + 1, r - 1, s),
//         (q + 1, r, s - 1),
//         (q, r + 1, s - 1),
//         (q - 1, r + 1, s),
//         (q - 1, r, s + 1),
//         (q, r - 1, s + 1),
//     ]
// }

// // Get 6 neighbouring tile co-ordinates in cube co-ordinates but as a BTreeset
// fn bench_neighbour_tiles(position: (i8, i8, i8)) -> BTreeSet<(i8, i8, i8)> {
//     let (q, r, s) = position;

//     BTreeSet::from([
//         (q + 1, r - 1, s),
//         (q + 1, r, s - 1),
//         (q, r + 1, s - 1),
//         (q - 1, r + 1, s),
//         (q - 1, r, s + 1),
//         (q, r - 1, s + 1),
//     ])
// }

// // Get co-ordinates of all chips that are already placed on the board
// fn get_placed_positions<T: Coord>(board: &Board<T>) -> Vec<(i8, i8, i8)> {
//     board.chips.values().flatten().copied().collect()
// }

// // Get co-ordinates of all chips that are already placed on the board
// fn bench_get_placed_positions<T: Coord>(board: &Board<T>) -> BTreeSet<(i8, i8, i8)> {
//     board.chips.values().flatten().copied().collect()
// }

// // Get co-ordinates of all chips that are already placed on the board
// fn bench2_get_placed_positions<T: Coord>(board: &Board<T>) -> HashSet<(i8, i8, i8)> {
//     board.chips.values().flatten().copied().collect()
// }

// // The silly way using two for loops, originally implemented in board
// fn count_neighbours<T: Coord>(board: &Board<T>, position: (i8, i8, i8)) -> usize {
//     // Count number of neighbouring chips

//     // Store neighbours here
//     let mut neighbours = HashSet::new();

//     // Get the co-ordinates of neighbouring hexes
//     let neighbour_hexes = neighbour_tiles(position);

//     // If any of these neighbouring hex coords also appear in the board's list of chips, it means they're a neighbouring chip
//     let chip_positions = get_placed_positions(board);

//     // Add common vector elements to the hashset using that terrible double-for loop
//     for elem in neighbour_hexes.iter() {
//         for elem2 in chip_positions.clone().iter() {
//             if (elem == elem2) & (!neighbours.contains(elem2)) {
//                 neighbours.insert(*elem2); // add the neighbour to the hashset
//             }
//         }
//     }

//     neighbours.len()
// }

// // Force both to become BTreeSets and use intersection
// fn count_neighbours_2<T: Coord>(board: &Board<T>, position: (i8, i8, i8)) -> usize {
//     // Count number of neighbouring chips. This is 4 times faster than using a double for loop.

//     // Get the co-ordinates of neighbouring hexes as btree
//     let neighbour_hexes = neighbour_tiles(position)
//         .into_iter()
//         .collect::<BTreeSet<_>>();

//     // If any of these neighbouring hex coords also appear in the board's list of chips, it means they're a neighbouring chip
//     let chip_positions = get_placed_positions(board)
//         .into_iter()
//         .collect::<BTreeSet<_>>();

//     // Try intersect and count
//     neighbour_hexes.intersection(&chip_positions).count()
// }

// // Get both as BTreesets first time round and use intersection
// fn count_neighbours_3<T: Coord>(board: &Board<T>, position: (i8, i8, i8)) -> usize {
//     // Count number of neighbouring chips. This is 4 times faster than using a double for loop.

//     // Get the co-ordinates of neighbouring hexes as btree
//     let neighbour_hexes = bench_neighbour_tiles(position);

//     // If any of these neighbouring hex coords also appear in the board's list of chips, it means they're a neighbouring chip
//     let chip_positions = bench_get_placed_positions(board);

//     // Try intersect and count
//     neighbour_hexes.intersection(&chip_positions).count()
// }

// // As above, but use filter instead of intersection method
// pub fn count_neighbours_4<T: Coord>(board: &Board<T>, position: (i8, i8, i8)) -> usize {
//     // Get the co-ordinates of neighbouring hexes as btree
//     let neighbour_hexes = bench_neighbour_tiles(position);

//     // If any of these neighbouring hex coords also appear in the board's list of chips, it means they're a neighbouring chip
//     let chip_positions = bench_get_placed_positions(board);

//     // Try a filter way and count
//     neighbour_hexes
//         .iter()
//         .filter(|v| chip_positions.contains(v))
//         .count()
// }

// // Do Hashset instead of BTRee
// pub fn count_neighbours_5<T: Coord>(board: &Board<T>, position: (i8, i8, i8)) -> usize {
//     // Get the co-ordinates of neighbouring hexes
//     let neighbour_hexes = neighbour_tiles(position);

//     // If any of these neighbouring hex coords also appear in the board's list of chips, it means they're a neighbouring chip
//     let chip_positions = bench2_get_placed_positions(board);

//     // Try a filter way and count
//     neighbour_hexes
//         .iter()
//         .filter(|v| chip_positions.contains(v))
//         .count()
// }

// fn criterion_benchmark(c: &mut Criterion) {
//     let board = setup_board();
//     let position = board.coord.mapfrom_doubleheight((0, -2));
//     c.bench_function("double_for_loop", |b| {
//         b.iter(|| count_neighbours(&board, position))
//     }); // slowest
//     c.bench_function("btree_intersect", |b| {
//         b.iter(|| count_neighbours_2(&board, position))
//     }); // 4x improvement
//     c.bench_function("btree_always", |b| {
//         b.iter(|| count_neighbours_3(&board, position))
//     }); // similar, one of the tidiest options
//     c.bench_function("btree_always_filter", |b| {
//         b.iter(|| count_neighbours_4(&board, position))
//     }); // same again
//     c.bench_function("hashset_always_filter", |b| {
//         b.iter(|| count_neighbours_5(&board, position))
//     }); // same again
// }

// criterion_group!(benches, criterion_benchmark);
// criterion_main!(benches);
