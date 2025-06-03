//! Implementation of search spaces and problems.
//!
//! These expose a generic search space so we can do pathfinding against a
//! generic graph-like API where from a given state we can find actions that
//! take us to new states.

pub mod maze_2d;
#[cfg(feature = "osm")]
pub mod osm;
