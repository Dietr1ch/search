use std::fmt::Debug;

use crate::space::Cost;
use crate::space::State;

use nonmax::NonMaxUsize;

#[derive(Debug)]
pub struct Node<St, C>
where
    St: State,
    C: Cost,
{
    pub(crate) parent: Option<St>,
    // pub(crate) state: St,
    pub(crate) g: C,
    pub(crate) heap_index: NonMaxUsize,
    pub(crate) is_closed: bool,
}

impl<St, C> Node<St, C>
where
    St: State,
    C: Cost,
{
    pub fn new(parent: Option<St>, g: C, heap_index: usize) -> Self {
				Self {
            parent,
            g,
            heap_index: NonMaxUsize::try_from(heap_index).unwrap(),
            is_closed: false,
				}
    }

    /// Gives this Node a better path through a new parent.
    pub fn reach(&mut self, new_parent: St, g: C) {
        debug_assert!(g < self.g);
        self.parent = Some(new_parent);
        self.g = g;
    }
}

