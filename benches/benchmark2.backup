// cargo benchmark

// This no longer works because changes based on findings were implemented to coord system

// We'll compare co-ordinate systems that use tuples for x,y,z with co-ordinate systems that use vec
// Vec has the advantage of mapping onto non 3d coordinate systems easily
// Conclusion - tuples 140 ns vs vecs 240 ns vs lists(slices) 140 ns vs vecdeq 264 ns.
// Tuples and slices are comparable. Stick with tuples but for 1 coord systems just ignore the second two items in the tuple.
// StructTuples are weirdly faster than regular tuples, and can impl a generic
// TODO: impl addition for the trait? optional really.

// use criterion::{criterion_group, criterion_main, Criterion};

// use std::collections::{HashSet, VecDeque};

// // Define vector version of storing and using cube co-ordinates.
// trait CoordVec{
//     fn neighbour_tiles(&self, position: Vec<i8>) -> HashSet<Vec<i8>>;
// }

// struct CubeVec;
// impl CoordVec for CubeVec{
//     fn neighbour_tiles(&self, p: Vec<i8>) -> HashSet<Vec<i8>> {
//         HashSet::from([
//             Vec::from([p[0] + 1, p[1]  - 1, p[2]]),
//             Vec::from([p[0] + 1, p[1],  p[2] - 1]),
//             Vec::from([p[0] , p[1] + 1,  p[2] - 1]),
//             Vec::from([p[0]  - 1, p[1] + 1,  p[2]]),
//             Vec::from([p[0]  - 1, p[1],  p[2] + 1]),
//             Vec::from([p[0] , p[1] - 1,  p[2] + 1]),
//         ])
//     }
// }

// // Define vector deque version of storing and using cube co-ordinates.
// trait CoordVecDeq{
//     fn neighbour_tiles(&self, position: VecDeque<i8>) -> HashSet<VecDeque<i8>>;
// }

// struct CubeVecDeq;
// impl CoordVecDeq for CubeVecDeq{
//     fn neighbour_tiles(&self, position: VecDeque<i8>) -> HashSet<VecDeque<i8>> {
//         let (q, r, s) = (position[0], position[1], position[2]);

//         HashSet::from([
//             VecDeque::from([q + 1, r - 1, s]),
//             VecDeque::from([q + 1, r, s - 1]),
//             VecDeque::from([q, r + 1, s - 1]),
//             VecDeque::from([q - 1, r + 1, s]),
//             VecDeque::from([q - 1, r, s + 1]),
//             VecDeque::from([q, r - 1, s + 1]),
//         ])
//     }
// }

// // Tuple version of storing and using cube coordinates.
// trait CoordTuple{
//     fn neighbour_tiles(&self, position: (i8,i8,i8)) -> HashSet<(i8,i8,i8)>;
// }

// struct CubeTuple;
// impl CoordTuple for CubeTuple {
//     fn neighbour_tiles(&self, position: (i8, i8, i8)) -> HashSet<(i8, i8, i8)> {
//         let (q, r, s) = position;

//         HashSet::from([
//             (q + 1, r - 1, s),
//             (q + 1, r, s - 1),
//             (q, r + 1, s - 1),
//             (q - 1, r + 1, s),
//             (q - 1, r, s + 1),
//             (q, r - 1, s + 1),
//         ])
//     }
// }

// trait Position: Hash + Eq + Clone + Copy + Add{
//     //fn nothing(&self);
//     fn new(x:i8,y:i8,z:i8) -> Self;
//     //fn add(self, other:Self) -> Self;
//     //fn toPosition(self) -> CubePosition;

// }

// #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
// struct CubePosition{
//     x: i8,
//     y: i8,
//     z: i8,
// }

// impl Position for CubePosition{
//     // fn nothing(&self){
//     // }
//     fn new(x:i8,y:i8,z:i8) -> Self {
//         CubePosition { x, y, z }
//     }

//     // fn toPosition(self) -> CubePosition {
//     //     self
//     // }
// }

// use std::ops::Add;
// impl Add for CubePosition{
//     type Output = Self;

//     fn add(self, other: Self) -> Self {
//         Self{
//             x: self.x + other.x,
//             y: self.y + other.y,
//             z: self.z + other.z,
//         }
//     }
// }

// use std::hash::Hash;
// trait CoordStructTuple{
//     fn neighbour_tiles<T: Position  + Add<Output = T>>(&self, position: T) -> HashSet<T>;
// }

// struct CubeStructTuple;
// impl CoordStructTuple for CubeStructTuple {
//     fn neighbour_tiles<T: Position + Add<Output = T>>(&self, position: T) -> HashSet<T> {

//         HashSet::from([
//             position + T::new(1, -1, 0),
//             position + T::new(1, 0, -1),
//             position + T::new(0, 1, -1),
//             position + T::new(-1, 1, 0),
//             position + T::new(-1, 0, 1),
//             position + T::new(0, -1, 1),
//         ])
//     }
// }

// // Define list version of storing and using cube co-ordinates.
// trait CoordList{
//     fn neighbour_tiles(&self, position: [Option<i8>;3]) -> HashSet<[i8;3]>;
// }

// struct CubeList;
// impl CoordList for CubeList{
//     fn neighbour_tiles(&self, position: [Option<i8>;3]) -> HashSet<[i8;3]> {

//         let q = position[0].unwrap();
//         let r = position[1].unwrap();
//         let s = position[2].unwrap();

//         HashSet::from([
//             [q + 1, r - 1, s],
//             [q + 1, r, s - 1],
//             [q, r + 1, s - 1],
//             [q - 1, r + 1, s],
//             [q - 1, r, s + 1],
//             [q, r - 1, s + 1],
//         ])
//     }
// }

// fn criterion_benchmark(c: &mut Criterion) {

//     let tupley = CubeTuple;
//     let veccy = CubeVec;
//     let listy = CubeList;
//     let vecdeqy = CubeVecDeq;
//     let structtupley = CubeStructTuple;

//     c.bench_function("using_tuples", |b| {
//         b.iter(|| tupley.neighbour_tiles((0,0,0)))
//     });

//     c.bench_function("using_structtuples", |b| {
//         b.iter(|| structtupley.neighbour_tiles(CubePosition::new(0,0,0)))
//     });

//     // c.bench_function("using_vecs", |b| {
//     //     b.iter(|| veccy.neighbour_tiles(vec![0,0,0]))
//     // });

//     c.bench_function("using_slices", |b| {
//         b.iter(|| listy.neighbour_tiles([Some(0),Some(0),Some(0)]))
//     });

//     // c.bench_function("using_vecdeq", |b| {
//     //     b.iter(|| vecdeqy.neighbour_tiles(VecDeque::from([0,0,0])))
//     // });
// }

// criterion_group!(benches, criterion_benchmark);
// criterion_main!(benches);
