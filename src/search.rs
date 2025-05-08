use std::fmt::Debug;

use crate::space::Action;
use crate::space::Cost;
use crate::space::Path;
use crate::space::Space;
use crate::space::State;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SearchTreeIndex(usize);

impl SearchTreeIndex {
    #[cfg(feature = "inspect")]
    pub fn fake_new() -> Self {
        SearchTreeIndex(0usize)
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "inspect", derive(Clone))]
pub struct SearchTreeNode<St, A, C>
where
    St: State,
    A: Action,
    C: Cost,
{
    pub(crate) parent: Option<(SearchTreeIndex, A)>,
    pub(crate) state: St,
    pub(crate) g: C,
    pub(crate) heap_index: usize,
}

impl<St, A, C> SearchTreeNode<St, A, C>
where
    St: State,
    A: Action,
    C: Cost,
{
    pub fn new(heap_index: usize, s: St, parent: Option<(SearchTreeIndex, A)>, g: C) -> Self {
        Self {
            parent,
            state: s,
            g,
            heap_index,
        }
    }

    /// Gives this Node a better path through a new parent.
    pub fn reach(&mut self, new_parent: (SearchTreeIndex, A), g: C) {
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
    pub(crate) fn state(&self) -> &St {
        &self.state
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "inspect", derive(Clone))]
pub(crate) struct SearchTree<St, A, C>
where
    St: State,
    A: Action,
    C: Cost,
{
    // TODO: Use an Arena as the SearchTreeNode store
    nodes: Vec<SearchTreeNode<St, A, C>>,
}

impl<St, A, C> SearchTree<St, A, C>
where
    St: State,
    A: Action,
    C: Cost,
{
    pub(crate) fn new() -> Self {
        Self { nodes: vec![] }
    }
    pub(crate) fn push(&mut self, node: SearchTreeNode<St, A, C>) -> SearchTreeIndex {
        let index = SearchTreeIndex(self.nodes.len());
        self.nodes.push(node);
        index
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    pub fn capacity(&self) -> usize {
        self.nodes.capacity()
    }

    pub fn path<Sp: Space<St, A, C>>(
        &mut self,
        space: &Sp,
        node_index: SearchTreeIndex,
    ) -> Path<St, A, C> {
        recover_path(space, &self.nodes, node_index)
    }
}

impl<St, A, C> Default for SearchTree<St, A, C>
where
    St: State,
    A: Action,
    C: Cost,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<St, A, C> std::ops::Index<SearchTreeIndex> for SearchTree<St, A, C>
where
    St: State,
    A: Action,
    C: Cost,
{
    type Output = SearchTreeNode<St, A, C>;

    fn index(&self, index: SearchTreeIndex) -> &Self::Output {
        &self.nodes[index.0]
    }
}

impl<St, A, C> std::ops::IndexMut<SearchTreeIndex> for SearchTree<St, A, C>
where
    St: State,
    A: Action,
    C: Cost,
{
    fn index_mut(&mut self, index: SearchTreeIndex) -> &mut SearchTreeNode<St, A, C> {
        &mut self.nodes[index.0]
    }
}

pub(crate) fn recover_path<Sp: Space<St, A, C>, St: State, A: Action, C: Cost>(
    space: &Sp,
    nodes: &[SearchTreeNode<St, A, C>],
    mut node_index: SearchTreeIndex,
) -> Path<St, A, C> {
    let e = &nodes[node_index.0];
    let mut path = Path::<St, A, C>::new_from_start(*e.state());

    while let Some((parent_index, a)) = nodes[node_index.0].parent {
        let p = &nodes[parent_index.0];
        let s = p.state();
        let c: C = space.cost(s, &a);
        debug_assert!(c != C::zero());

        path.append((*s, a), c);
        debug_assert!(node_index != parent_index);
        node_index = parent_index;
    }

    path.reverse();
    path
}
