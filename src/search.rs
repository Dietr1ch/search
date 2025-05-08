use std::fmt::Debug;

use nonmax::NonMaxUsize;

use crate::space::Action;
use crate::space::Cost;
use crate::space::Path;
use crate::space::Space;
use crate::space::State;

#[derive(Debug)]
#[cfg_attr(feature = "inspect", derive(Clone))]
pub struct SearchTreeNode<St, A, C>
where
    St: State,
    A: Action,
    C: Cost,
{
    pub parent: Option<(NonMaxUsize, A)>,
    pub state: St,
    pub g: C,
    pub heap_index: usize,
}

impl<St, A, C> SearchTreeNode<St, A, C>
where
    St: State,
    A: Action,
    C: Cost,
{
    pub fn new(heap_index: usize, s: St, g: C) -> Self {
        Self {
            parent: None,
            state: s,
            g,
            heap_index,
        }
    }
    pub fn new_from_parent(heap_index: usize, s: St, parent: (NonMaxUsize, A), g: C) -> Self {
        Self {
            parent: Some(parent),
            state: s,
            g,
            heap_index,
        }
    }

    pub fn reach(&mut self, new_parent: (NonMaxUsize, A), g: C) {
        debug_assert!(g < self.g);
        self.parent = Some(new_parent);
        self.g = g;
    }
}

impl<St, A, C> SearchTreeNode<St, A, C>
where
    St: State,
    A: Action,
    C: Cost,
{
    pub fn state(&self) -> &St {
        &self.state
    }
}

pub fn recover_path<Sp: Space<St, A, C>, St: State, A: Action, C: Cost>(
    space: &Sp,
    nodes: &[SearchTreeNode<St, A, C>],
    mut node_index: usize,
) -> Path<St, A, C> {
    let e = &nodes[node_index];
    let mut path = Path::<St, A, C>::new_from_start(*e.state());

    while let Some((parent_index, a)) = nodes[node_index].parent {
        let p = &nodes[parent_index.get()];
        let s = p.state();
        let c: C = space.cost(s, &a);
        debug_assert!(c != C::zero());

        path.append((*s, a), c);
        debug_assert!(node_index != parent_index.get());
        node_index = parent_index.get();
    }

    path.reverse();
    path
}
