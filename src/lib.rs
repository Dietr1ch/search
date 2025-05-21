#![feature(cmp_minmax)]
#![feature(core_intrinsics)]
#![feature(non_null_from_ref)]

use shadow_rs::shadow;

shadow!(build);

// Internals
// ---------
pub mod data_structures;
pub mod debug;
pub mod derank;
pub mod heap_primitives;

// Renderer
#[cfg(feature = "renderer")]
pub mod renderer;

// Search space and problems
// -------------------------
pub mod problem;
pub mod search;
pub mod space;

// Problems
pub mod problems;

// Algorithms
pub mod algorithms;
