use std::fmt::Debug;

use rustc_hash::FxHashMap;

use crate::heap_primitives::index_down_left;
use crate::heap_primitives::index_down_right;
use crate::heap_primitives::index_up;
use crate::problem::Problem;
use crate::search::SearchTree;
use crate::search::SearchTreeIndex;
use crate::search::SearchTreeNode;
use crate::space::Action;
use crate::space::Cost;
use crate::space::Path;
use crate::space::Space;
use crate::space::State;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DijkstraRank<C: Cost> {
    g: C,
}
impl<C> DijkstraRank<C>
where
    C: Cost,
{
    pub fn new(g: C) -> Self {
        Self { g }
    }
}

// TODO: Make public only with the "inspect" feature
#[derive(Debug)]
#[cfg_attr(feature = "inspect", derive(Clone))]
pub struct DijkstraHeapNode<C>
where
    C: Cost,
{
    pub rank: DijkstraRank<C>,
    /// The index of this node in the Node Arena
    pub node_index: SearchTreeIndex,
}

use std::marker::PhantomData;

#[derive(Debug)]
pub struct DijkstraSearch<P, Sp, St, A, C>
where
    P: Problem<Sp, St, A, C>,
    Sp: Space<St, A, C>,
    St: State,
    A: Action,
    C: Cost,
{
    /// All the Search Nodes. Naturally forms a Search Forest as each node may
    /// have a parent Node.
    ///
    /// Could be backed by an Arena since this collection only grows and does
    /// not need contiguous memory.
    search_tree: SearchTree<St, A, C>,

    /// An intrusive heap of (AStarRank, SearchTreeIndex) that keeps the
    /// referenced node updated (SearchTreeNode::heap_index).
    /// This allows re-ranking a SearchTreeNode in the heap without a linear
    /// search for its (AStarRank, SearchTreeIndex) entry.
    ///
    /// for (i, hn) in self.open.enumerate():
    ///   assert_eq(self.search_tree[hn.node_index].heap_index, i)
    open: Vec<DijkstraHeapNode<C>>,

    /// Amalgamation of,
    /// - The `HashMap<St, &mut SearchTreeNode>`, but using SearchTreeIndex
    ///   - To find existing Search Nodes from their State.
    /// - The "Closed Set" `HashSet<St>`
    ///   - To recall whether we had already explored a state.
    ///
    /// It's the same size as the Search Tree.
    node_map: FxHashMap<St, SearchTreeIndex>,

    problem: P,

    // TODO: Clean PhantomData if possible.
    _phantom_space: PhantomData<Sp>,
    _phantom_action: PhantomData<A>,
}

impl<P, Sp, St, A, C> DijkstraSearch<P, Sp, St, A, C>
where
    P: Problem<Sp, St, A, C>,
    Sp: Space<St, A, C>,
    St: State,
    A: Action,
    C: Cost,
{
    #[must_use]
    pub fn new(p: P) -> Self {
        let mut search = Self {
            search_tree: SearchTree::<St, A, C>::new(),
            open: vec![],
            node_map: FxHashMap::default(),

            problem: p,

            // TODO: Clean PhantomData
            _phantom_space: PhantomData,
            _phantom_action: PhantomData,
        };

        for s in search.problem.starts().clone() {
            let g: C = C::zero();
            let parent: Option<(SearchTreeIndex, A)> = None;
            search.push_new(&s, parent, g);
        }

        search
    }

    #[must_use]
    pub fn find_next_goal(&mut self) -> Option<Path<St, A, C>> {
        // Check remaining un-explored nodes
        // NOTE: We could avoid a Heap::pop() by peeking and doing the goal-check.
        // TODO: See if pop_node() would be the same or faster
        while let Some(node_index) = self.pop() {
            let state = *self.search_tree[node_index].state();
            let g: C = self.search_tree[node_index].g;
            debug_assert!(!self.is_closed(&state));

            // NOTE: We can do a goal-check and return here if we only need one
            // path or can yield a result

            // Mark as closed
            self.mark_closed(&state);

            // Expand state
            for (s, a) in self.problem.space().neighbours(&state) {
                // Have we seen this State?
                match self.node_map.get(&s) {
                    Some(neigh_index) => {
                        if neigh_index.is_closed() {
                            // Yes, and we expanded the State already.
                            // NOTE: Could be a goal we had already found through a
                            // sub-optimal path. Currently we only search for
                            // an optimal path to a new goal.
                            continue;
                        }
                        // Yes, but it's still unexplored. Update the existing
                        // Node if needed.
                        let neigh = &mut self.search_tree[*neigh_index];
                        let neigh_heap_index = neigh.heap_index;
                        let c: C = self.problem.space().cost(&s, &a);
                        let new_g = g + c;
                        if new_g < neigh.g {
                            // Found better path to existing node
                            neigh.reach((node_index, a), new_g);
                            self.open[neigh.heap_index].rank = DijkstraRank::new(neigh.g);
                            self._unsafe_sift_up(neigh_heap_index);
                        }
                    }
                    None => {
                        // No, let's create a new Node for it.
                        let c: C = self.problem.space().cost(&s, &a);
                        let new_g = g + c;

                        self.push_new(&s, Some((node_index, a)), new_g);
                    }
                }
            }

            // NOTE: This should be done before expanding if we could yield or
            // only want the path to the first goal.
            if self.problem.is_goal(&state) {
                return Some(self.search_tree.path(self.problem.space(), node_index));
            }
        }

        None
    }

    #[inline(always)]
    #[must_use]
    pub(crate) fn is_closed(&self, s: &St) -> bool {
        match self.node_map.get(s) {
            Some(node_index) => node_index.is_closed(),
            None => false,
        }
    }
    #[inline(always)]
    fn mark_closed(&mut self, s: &St) {
        match self.node_map.get(s) {
            Some(node_index) => {
                if node_index.is_closed() {
                    // Closed a closed state
                    return;
                } else {
                    self.node_map.insert(*s, node_index.as_closed());
                }
            }
            None => {
                unreachable!("Tried closing a state without a node");
            }
        }
    }

    #[inline(always)]
    #[must_use]
    pub fn pop_node(&mut self) -> Option<&mut SearchTreeNode<St, A, C>> {
        match self.pop() {
            Some(i) => Some(&mut self.search_tree[i]),
            None => None,
        }
    }

    #[inline(always)]
    #[must_use]
    fn pop(&mut self) -> Option<SearchTreeIndex> {
        match self.open.len() {
            0 | 1 => self.open.pop().map(|n| n.node_index),
            _ => {
                self.verify_heap();
                let node_index = self._unsafe_pop_non_trivial_heap();
                self.verify_heap();
                Some(node_index)
            }
        }
    }

    #[inline(always)]
    fn push_new(&mut self, s: &St, parent: Option<(SearchTreeIndex, A)>, g: C) {
        self.verify_heap();
        debug_assert!(!self.is_closed(s));

        // NOTE: search_tree and open have indices to each other.
        // Compute next heap index to allow creating SearchTreeNode
        let heap_index = self.open.len(); // Future heap_index

        // 1. Add SearchTreeNode to search_tree
        let node_index: SearchTreeIndex = self
            .search_tree
            .push(SearchTreeNode::<St, A, C>::new(heap_index, *s, parent, g));
        let node = &mut self.search_tree[node_index];
        debug_assert_eq!(node.heap_index, heap_index);
        debug_assert_eq!(node.g, g);

        // 2. Add entry to node_map
        debug_assert!(!node_index.is_closed());
        self.node_map.insert(*s, node_index);

        // 3. Add AStarHeapNode to open using it's SearchTreeIndex
        self.open.push(DijkstraHeapNode {
            rank: DijkstraRank::new(g),
            node_index,
        });
        self._unsafe_sift_up(heap_index);

        self.verify_heap();
    }

    #[inline(always)]
    #[cfg(not(feature = "inspect"))]
    pub(crate) fn verify_heap(&self) {
        // All good... (hopefully)
    }
    #[inline(always)]
    #[cfg(feature = "inspect")]
    pub(crate) fn verify_heap(&self) {
        // Every node,
        for (i, e) in self.open.iter().enumerate() {
            // - Has the right intrusive index set.
            debug_assert!(self.search_tree[e.node_index].heap_index == i);

            // - Goes after its parent node, if any.
            if i == 0 {
                continue;
            }
            let p = index_up(i);
            debug_assert!(
                self.open[p].rank <= self.open[i].rank,
                "Node[{p}]={:?} !<= child [{i}]={:?}. Out of heap of len={}",
                self.open[p],
                self.open[i],
                self.open.len(),
            );
        }
    }

    /// Pops the top node from a Heap with at least 2 elements.
    ///
    /// Works by unfairly sifting down the top-node to the last level, where it can
    /// be swapped with the very last element of the array and popped
    /// Temporarily breaks invariants around the node sifting down unfairly.
    fn _unsafe_pop_non_trivial_heap(&mut self) -> SearchTreeIndex {
        debug_assert!(!self.open.is_empty(), "You can't pop from an empty heap");
        debug_assert!(
            self.open.len() != 1,
            "It doesn't get easier. Why are you calling this?"
        );

        // Heap 101
        //
        //                            0
        //              1                           2
        //       3            4              5             6
        //   7      8      9     10      11     12     13     14
        // 15 16  17 18  19 20  21 22  23  24  25 !   *  *   *  *
        //
        // The last level WILL OFTEN be incomplete
        //
        //   - Up: (i-1)//2
        //   - DL: (2*i) + 1
        //   - DR: 2(i+1)

        // There's at least 2 nodes before we remove the best.
        // 1. We pretend there's a hole at the root, and bubble elements up till the hole reaches the bottom.
        // 2. If the hole is not the last element, we swap it for the last one.
        // 3. Now the last element is the one that was at the top of the heap, we pop it.

        // Initialize bubble-down indices
        let mut hole = 0;
        let mut child = index_down_left(hole); // Initially left child, reused to track the best child
        debug_assert!(hole < self.open.len(), "The hole IS NOT a valid index");
        debug_assert!(child < self.open.len(), "Left child IS NOT a valid index");
        debug_assert!(hole < child);
        let last = self.open.len() - 1;

        loop {
            debug_assert!(hole < self.open.len(), "The hole IS NOT a valid index");
            debug_assert!(child < self.open.len(), "Left child IS NOT a valid index");
            // Find the best child
            let child_r = child + 1;
            debug_assert_eq!(child_r, index_down_right(hole));
            if child_r < self.open.len() && self.open[child_r].rank < self.open[child].rank {
                child = child_r;
            }

            // Swap and update internal indices
            self._unsafe_half_swap_down(hole, child);

            // Update bubble-down indices
            hole = child;
            child = index_down_left(hole); // New left child
            if child >= self.open.len() {
                break;
            }
        }
        // NOTE: So far the hole made it to the last level, but it may not be at the end of the array.
        debug_assert!(hole <= last, "The hole={hole} is < last={last}");
        debug_assert!(hole > index_up(last), "The hole={hole} is < last={last}");
        if hole != last {
            // Swap and update internal indices
            self._unsafe_half_swap_down(hole, last);
            self._unsafe_sift_up(hole);
        }

        let heap_node = self.open.pop().unwrap();
        debug_assert_eq!(
            self.search_tree[heap_node.node_index].heap_index, 0,
            "Top node half-assed swapped down should still have it's 0 index"
        );

        heap_node.node_index
    }

    /// Raises a node
    #[inline(always)]
    fn _unsafe_sift_up(&mut self, index: usize) -> usize {
        debug_assert!(
            index < self.open.len(),
            "Node is way out of sync. Index out of bounds..."
        );
        debug_assert!(
            self.search_tree[self.open[index].node_index].heap_index == index,
            "Node is out of sync."
        );

        // Can't improve
        if index == 0 {
            return index;
        }

        let mut pos = index;
        let mut parent = index_up(pos);
        while self.open[parent].rank > self.open[pos].rank {
            // Nodes are different and swapped. Swap the nodes to fix the order.
            self._unsafe_swap(parent, pos);
            debug_assert!(self.open[parent].rank < self.open[pos].rank);

            // Continue swapping upwards if needed..
            if parent == 0 {
                return parent;
            }
            pos = parent;
            parent = index_up(pos);
        }
        pos
    }

    // Swapping primitives
    /// Swaps two elements in the heap.
    ///
    /// For consistency in calling code l < r is checked.
    ///
    /// Keeps the intrusive indices in sync.
    #[inline(always)]
    fn _unsafe_swap(&mut self, l: usize, r: usize) {
        debug_assert!(l < r, "Swap({l}, {r}) uses wrong argument order");

        let len = self.open.len();
        debug_assert!(l < len, "Left  swap index {} is OUT OF BOUNDS({})", l, len);
        debug_assert!(r < len, "Right swap index {} is OUT OF BOUNDS({})", r, len);
        self.open.swap(l, r);
        self.search_tree[self.open[l].node_index].heap_index = l;
        self.search_tree[self.open[r].node_index].heap_index = r;
        debug_assert!(
            self.open[l].rank <= self.open[r].rank,
            "Swaps must locally restore the heap invariant."
        );
    }
    /// Swaps two elements in the heap.
    ///
    /// For consistency in calling code l < r is checked.
    ///
    /// Only keeps the index of the element going up in sync as we should shortly
    /// after remove the element that goes down.
    #[inline(always)]
    fn _unsafe_half_swap_down(&mut self, l: usize, r: usize) {
        debug_assert!(l < r, "HalfSwapDown({l}, {r}) is wrong");

        let len = self.open.len();
        debug_assert!(l < len, "Left  swap index {} is OUT OF BOUNDS({})", l, len);
        debug_assert!(r < len, "Right swap index {} is OUT OF BOUNDS({})", r, len);
        self.open.swap(l, r);
        self.search_tree[self.open[l].node_index].heap_index = l;
        debug_assert!(
            self.open[l].rank >= self.open[r].rank, // (=? What if there's only one value? We still push node at the top down)
            "Half-assed swap down must be unfairly pushing a node down."
        );
        debug_assert!(
            self.search_tree[self.open[r].node_index].heap_index < r,
            "Node half-assed swapped down should still point to it's original index."
        );
    }

    pub fn print_memory_stats(&self) {
        use std::mem::size_of;

        println!("DijkstraSearch Stats:");
        let s = size_of::<SearchTreeNode<St, A, C>>();
        let l = self.search_tree.len();
        println!("  - |Nodes|:   {} ({}B)", l, l * s);

        let s = size_of::<DijkstraHeapNode<C>>();
        let l = self.open.len();
        let c = self.open.capacity();
        println!("  - |Open|:   {} ({}B)", l, l * s);
        println!("  - |Open|*:  {} ({}B)", c, c * s);

        let s = size_of::<(St, SearchTreeIndex)>();
        let l = self.node_map.len();
        let c = self.node_map.capacity();
        println!("  - |Index|:  {} ({}B)", l, l * s);
        println!("  - |Index|*: {} ({}B)", c, c * s);
    }
}

impl<P, Sp, St, A, C> Iterator for DijkstraSearch<P, Sp, St, A, C>
where
    P: Problem<Sp, St, A, C>,
    Sp: Space<St, A, C>,
    St: State,
    A: Action,
    C: Cost,
{
    type Item = Path<St, A, C>;
    fn next(&mut self) -> Option<Self::Item> {
        self.find_next_goal()
    }
}
