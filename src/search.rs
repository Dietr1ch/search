use std::fmt::Debug;

use typed_arena::Arena;

use crate::space::Action;
use crate::space::Cost;
use crate::space::Path;
use crate::space::Space;
use crate::space::State;

/// The least-significant bit.
const LEAST_SIGNIFICANT_BIT: usize = 1usize;
/// The bit used to track `is_closed: bool` within pointers.
///
/// We abuse alignment of `SearchTreeNode` to sneak a `bool` into our pointers
/// (`SearchTreeIndex`) to them. This is guaranteed to be free since
/// `SearchTreeNode::<St, A, C>::heap_index` makes the type wider than a Byte
/// already.
const IS_CLOSED_BIT: usize = LEAST_SIGNIFICANT_BIT;

/// A reference to a `SearchTreeNode<St, A, C>`.
///
/// It's more like a `(&SearchTreeNode<St, A, C>, bool)` underneath to help track
/// whether the node is closed.
///
/// `ointers` generalises using the unnecessary bits in a pointer, but offers
/// them in a buffer and is still the same native pointer width, so
/// `(ointers::Ptr<T>, bool)` still uses more bits than `ointers::Ptr<T>`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SearchTreeIndex {
    index: usize,
}

impl SearchTreeIndex {
    #[inline(always)]
    fn new(index: usize) -> Self {
        Self { index }
    }
    #[cfg(feature = "inspect")]
    #[inline(always)]
    pub fn fake_new() -> Self {
        Self::new(0usize)
    }

    pub fn is_closed(&self) -> bool {
        self.index & IS_CLOSED_BIT == IS_CLOSED_BIT
    }
    pub fn set_closed(&mut self) {
        debug_assert!(!self.is_closed());
        self.index |= IS_CLOSED_BIT;
    }

    #[inline(always)]
    fn from_ptr<St: State, A: Action, C: Cost>(ptr: *const SearchTreeNode<St, A, C>) -> Self {
        let i = Self::new(ptr as usize);
        debug_assert!(!i.is_closed());
        i
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

pub(crate) struct SearchTree<St, A, C>
where
    St: State,
    A: Action,
    C: Cost,
{
    nodes: Arena<SearchTreeNode<St, A, C>>,
}

impl<St, A, C> SearchTree<St, A, C>
where
    St: State,
    A: Action,
    C: Cost,
{
    #[inline(always)]
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            nodes: Arena::<SearchTreeNode<St, A, C>>::new(),
        }
    }

    #[inline(always)]
    pub(crate) fn push(&mut self, node: SearchTreeNode<St, A, C>) -> SearchTreeIndex {
        let node = self.nodes.alloc(node);
        SearchTreeIndex::from_ptr::<St, A, C>(node as *const _)
    }

    #[inline(always)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    pub fn path<Sp: Space<St, A, C>>(
        &mut self,
        space: &Sp,
        mut node_index: SearchTreeIndex,
    ) -> Path<St, A, C> {
        #[cfg(feature = "coz_profile")]
        coz::scope!("PathReconstruction");

        let e = &self[node_index];
        let mut path = Path::<St, A, C>::new_from_start(*e.state());

        while let Some((parent_index, a)) = self[node_index].parent {
            let p = &self[parent_index];
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
}

impl<St, A, C> Default for SearchTree<St, A, C>
where
    St: State,
    A: Action,
    C: Cost,
{
    #[inline(always)]
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

    #[inline(always)]
    fn index(&self, index: SearchTreeIndex) -> &Self::Output {
        // TODO: Wrap this into something slightly safer
        unsafe {
            let index = index.index & !IS_CLOSED_BIT;
            &*(index as *mut SearchTreeNode<St, A, C>)
        }
    }
}

impl<St, A, C> std::ops::IndexMut<SearchTreeIndex> for SearchTree<St, A, C>
where
    St: State,
    A: Action,
    C: Cost,
{
    #[inline(always)]
    fn index_mut(&mut self, index: SearchTreeIndex) -> &mut SearchTreeNode<St, A, C> {
        // TODO: Wrap this into something slightly safer
        unsafe {
            let index = index.index & !IS_CLOSED_BIT;
            &mut *(index as *mut SearchTreeNode<St, A, C>)
        }
    }
}

impl<St, A, C> std::fmt::Debug for SearchTree<St, A, C>
where
    St: State,
    A: Action,
    C: Cost,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "SearchTree{{({} nodes)}}", self.len())
    }
}
