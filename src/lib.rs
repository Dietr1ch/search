#![feature(cmp_minmax)]
#![feature(non_null_from_ref)]

use shadow_rs::shadow;

shadow!(build);

pub mod debug;
pub mod heap_primitives;
pub mod maze_2d;
pub mod problem;
pub mod search;
pub mod space;

pub mod astar;
pub mod dijkstra;
