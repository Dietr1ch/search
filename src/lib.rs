#![feature(cmp_minmax)]
#![feature(non_null_from_ref)]

use shadow_rs::shadow;

shadow!(build);

pub mod debug;
pub mod heap_primitives;
pub mod heuristic_search;
pub mod maze_2d;
pub mod search;
pub mod space;
