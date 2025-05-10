#![feature(cmp_minmax)]
#![feature(non_null_from_ref)]

use shadow_rs::shadow;

shadow!(build);

// Internals
// ---------
pub mod debug;
pub mod heap_primitives;

// Search space and problems
// -------------------------
pub mod problem;
pub mod search;
pub mod space;

// Problems
// --------
pub mod maze_2d;

// Algorithms
// ----------
pub mod astar;
pub mod dijkstra;
