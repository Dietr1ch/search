//! Implementation of Dijkstra's path-finding algorithm.

use core::intrinsics::unlikely;
use std::cmp::min;
use std::fmt::Debug;
use std::marker::PhantomData;

use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;

use crate::derank::derank;
use crate::problem::ObjectiveProblem;
use crate::search::SearchTree;
use crate::search::SearchTreeIndex;
use crate::search::SearchTreeNode;
use crate::space::Action;
use crate::space::Cost;
use crate::space::Path;
use crate::space::Space;
use crate::space::State;

/// The ranking value for Dijkstra
///
/// We prefer better g-values.
///
// ```
// use search::algorithms::dijkstra::DijkstraRank;
// use search::space::Cost;
//
// let l0 = LittleCost::new(0);
// let l1 = LittleCost::new(1);
// assert!(DijkstraRank::new(l0) < DijkstraRank::new(l1));
// ```
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
    /// Improves `g`
    pub fn improve_g(&mut self, new_g: C) {
        debug_assert!(self.g > new_g);
        self.g = new_g;
    }
}

const HEAP_ARITY: usize = 8usize;
#[inline(always)]
#[must_use]
fn up(i: usize) -> usize {
    crate::heap_primitives::index_parent::<HEAP_ARITY>(i)
}
#[inline(always)]
#[must_use]
fn down_left(i: usize) -> usize {
    crate::heap_primitives::index_first_children::<HEAP_ARITY>(i)
}
#[inline(always)]
#[must_use]
fn down_right(i: usize) -> usize {
    crate::heap_primitives::index_last_children::<HEAP_ARITY>(i)
}

/// A heap node for Dijkstra
///
/// Heap nodes carry just ranking information and a reference/index to the
/// actual search nodes. This allows heap operations to move as little data as
/// possible.
// TODO: Make public only with the "inspect" feature
#[derive(Debug)]
#[cfg_attr(feature = "inspect", derive(Clone))]
pub struct DijkstraHeapNode<C>
where
    C: Cost,
{
    /// The rank of this node that defines how good it is.
    pub rank: DijkstraRank<C>,
    /// The index of this node in the Node Arena. Ignored when ranking.
    pub node_index: SearchTreeIndex,
}

/// `PartialEq` is forwarded to `self.rank`
impl<C: Cost> PartialEq for DijkstraHeapNode<C> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.rank.eq(&other.rank)
    }
}
/// `Eq` just says our `PartialEq` is also reflexive (`∀a. a==a`).
/// `https://doc.rust-lang.org/std/cmp/trait.Eq.html`
impl<C: Cost> Eq for DijkstraHeapNode<C> {}

/// `PartialOrd` is forwarded to `Ord::cmp`
impl<C: Cost> PartialOrd for DijkstraHeapNode<C> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.rank.cmp(&other.rank))
    }
}
/// `Ord` is forwarded to `self.rank`
impl<C: Cost> Ord for DijkstraHeapNode<C> {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rank.cmp(&other.rank)
    }
}

/// Dijkstra search implementation for Objective Problems.
///
/// This initializes the search and offers an Iterator that goes around
/// different solutions.
#[derive(Debug)]
pub struct DijkstraSearch<OP, Sp, St, A, C>
where
    OP: ObjectiveProblem<Sp, St, A, C>,
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

    /// An intrusive heap of `(DijkstraRank, SearchTreeIndex)` that keeps the
    /// referenced node updated (`SearchTreeNode::heap_index`).
    /// This allows re-ranking a `SearchTreeNode` in the heap without a linear
    /// search for its `(DijkstraRank, SearchTreeIndex)` entry.
    ///
    /// ```pseudocode
    /// for (i, hn) in self.open.enumerate():
    ///   assert_eq(self.search_tree[hn.node_index].heap_index, i)
    /// ```
    open: Vec<DijkstraHeapNode<C>>,

    /// Amalgamation of,
    /// - The `HashMap<St, &mut SearchTreeNode>`, but using `SearchTreeIndex`
    ///   - To find existing Search Nodes from their `State`.
    /// - The "Closed Set" `HashSet<St>`
    ///   - To recall whether we had already explored a state.
    ///
    /// It's the same size as the Search Tree.
    node_map: FxHashMap<St, SearchTreeIndex>,

    /// Set of remaining goal states.
    ///
    /// Used to cut the search earlier.
    remaining_goals_set: FxHashSet<St>,

    problem: OP,

    _phantom_space: PhantomData<Sp>,
    _phantom_action: PhantomData<A>,
}

impl<OP, Sp, St, A, C> DijkstraSearch<OP, Sp, St, A, C>
where
    OP: ObjectiveProblem<Sp, St, A, C>,
    Sp: Space<St, A, C>,
    St: State,
    A: Action,
    C: Cost,
{
    /// Initializes the Search
    #[must_use]
    pub fn new(op: OP) -> Self {
        let starts = op.starts().to_vec();
        let goals = op.goals().to_vec();

        let mut search = Self {
            search_tree: SearchTree::<St, A, C>::new(),
            open: Vec::with_capacity(2048),
            node_map: FxHashMap::default(),
            remaining_goals_set: FxHashSet::from_iter(goals.iter().cloned()),

            problem: op,

            _phantom_space: PhantomData,
            _phantom_action: PhantomData,
        };

        for s in starts {
            let g: C = C::zero();
            let parent: Option<(SearchTreeIndex, A)> = None;
            search.push_new(&s, parent, g);
        }

        search
    }

    /// Runs the search until the first goal is found.
    #[must_use]
    pub fn find_next_goal(&mut self) -> Option<Path<St, A, C>> {
        #[cfg(feature = "coz_profile")]
        coz::scope!("FindNextGoal");

        if self.remaining_goals_set.is_empty() {
            return None;
        }

        // Check remaining un-explored nodes
        // NOTE: We could avoid a `Heap::pop()` by peeking and doing the goal-check.
        // TODO: See if `pop_node()` would be the same or faster that `pop()`
        while let Some(node_index) = self.pop() {
            #[cfg(feature = "coz_profile")]
            coz::scope!("NodeExpansion");

            let state = *self.search_tree[node_index].state();
            let g: C = self.search_tree[node_index].g;
            debug_assert!(!self.is_closed(&state));

            // NOTE: We can do a goal-check and return here if we only need one
            // path or can yield a result

            // Mark as closed
            self.mark_closed(&state);

            // Expand state
            for (s, a) in self.problem.space().neighbours(&state) {
                #[cfg(feature = "coz_profile")]
                coz::scope!("ReachNode");

                // Have we seen this State?
                match self.node_map.get(&s) {
                    Some(neigh_index) => {
                        #[cfg(feature = "coz_profile")]
                        coz::scope!("ReachExistingNode");
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
                            self.open[neigh_heap_index].rank.improve_g(new_g);
                            self._unsafe_sift_up(neigh_heap_index);
                        }
                    }
                    None => {
                        #[cfg(feature = "coz_profile")]
                        coz::scope!("ReachNewNode");
                        // No, let's create a new Node for it.
                        let c: C = self.problem.space().cost(&s, &a);
                        let new_g = g + c;

                        self.push_new(&s, Some((node_index, a)), new_g);
                    }
                }
            }

            // NOTE: This should be done before expanding if we could yield or
            // only want the path to the first goal.
            if unlikely(self.is_goal(&state)) {
                #[cfg(feature = "coz_profile")]
                coz::progress!("GoalFound");
                self.remove_goal(&state);
                return Some(self.search_tree.path(self.problem.space(), node_index));
            }
        }

        None
    }

    /// Checks if a state is an undiscovered goal.
    #[inline(always)]
    fn is_goal(&mut self, s: &St) -> bool {
        self.remaining_goals_set.contains(s)
    }

    /// Removes a state from the remaining goals.
    #[inline(always)]
    fn remove_goal(&mut self, goal: &St) {
        #[cfg(feature = "coz_profile")]
        coz::scope!("RemoveGoal");

        // Remove the goal from the remaining goal set.
        self.remaining_goals_set.remove(goal);
    }

    /// Checks if a Search Node is already Closed (was expanded and explored)
    #[inline(always)]
    #[must_use]
    fn is_closed(&self, s: &St) -> bool {
        match self.node_map.get(s) {
            Some(node_index) => node_index.is_closed(),
            None => false,
        }
    }

    /// Marks a Search Node as Closed (expanded)
    #[inline(always)]
    fn mark_closed(&mut self, s: &St) {
        match self.node_map.get_mut(s) {
            Some(node_index) => {
                if !node_index.is_closed() {
                    node_index.set_closed();
                }
            }
            None => {
                unreachable!("Tried closing a state without a node");
            }
        }
    }

    /// Pops a node from the Heap, returning its SearchTree index.
    #[inline(always)]
    #[must_use]
    fn pop(&mut self) -> Option<SearchTreeIndex> {
        #[cfg(feature = "coz_profile")]
        coz::scope!("Pop");

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

        // NOTE: `search_tree` and open have indices to each other.
        // Compute next heap index to allow creating `SearchTreeNode`
        let heap_index = self.open.len(); // Future `heap_index`

        // 1. Add `SearchTreeNode` to `search_tree`
        let node_index: SearchTreeIndex = self
            .search_tree
            .push(SearchTreeNode::<St, A, C>::new(heap_index, *s, parent, g));
        let node = &mut self.search_tree[node_index];
        debug_assert_eq!(node.heap_index, heap_index);
        debug_assert_eq!(node.g, g);

        // 2. Add entry to `node_map`
        debug_assert!(!node_index.is_closed());
        self.node_map.insert(*s, node_index);

        // 3. Add `DijkstraHeapNode` to open using it's `SearchTreeIndex`
        self.open.push(DijkstraHeapNode {
            rank: DijkstraRank::new(g),
            node_index,
        });
        self._unsafe_sift_up(heap_index);

        self.verify_heap();
    }

    #[inline(always)]
    #[cfg(not(feature = "verify"))]
    pub(crate) fn verify_heap(&self) {
        // All good... (hopefully)
    }
    #[inline(always)]
    #[cfg(feature = "verify")]
    pub(crate) fn verify_heap(&self) {
        // Every node,
        for (i, e) in self.open.iter().enumerate() {
            // - Has the right intrusive index set.
            debug_assert!(self.search_tree[e.node_index].heap_index == i);

            // - Goes after its parent node, if any.
            if i == 0 {
                continue;
            }
            let p = up(i);
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
        #[cfg(feature = "coz_profile")]
        coz::scope!("PopNonTrivial");

        debug_assert!(!self.open.is_empty(), "You can't pop from an empty heap");
        debug_assert!(
            self.open.len() != 1,
            "It doesn't get easier. Why are you calling this?"
        );

        // Note that there's at least 2 nodes before we remove the best.
        // 1. We pretend there's a hole at the root, and bubble elements up till the hole reaches the bottom.
        // 2. If the hole is not the last element, we swap it for the last one.
        // 3. Now the last element is the one that was at the top of the heap, we pop it.

        let len = self.open.len();
        let last = len - 1;

        // Initialize bubble-down indices
        let mut hole = 0;
        let mut child = down_left(hole); // Initially left child, reused to track the best child
        debug_assert!(hole < len, "The hole IS NOT a valid index");
        debug_assert!(child < len, "Left child IS NOT a valid index");
        debug_assert!(hole < child);

        loop {
            debug_assert!(hole < len, "The hole IS NOT a valid index");
            debug_assert!(child < len, "Left child IS NOT a valid index");

            // Find the best child
            child = down_left(hole);
            debug_assert_eq!(child + HEAP_ARITY, down_right(hole) + 1);
            child += derank(&self.open[child..min(child + HEAP_ARITY, len)]);

            debug_assert!(self.open[hole].rank <= self.open[child].rank);

            // Swap and update internal indices
            self._unsafe_half_swap_down(hole, child);

            // Update bubble-down indices
            hole = child;
            child = down_left(hole); // New left child
            if child >= self.open.len() {
                break;
            }
        }
        // NOTE: So far the hole made it to the last level, but it may not be at the end of the array.
        debug_assert!(hole <= last, "The hole={hole} is < last={last}");
        debug_assert!(hole > up(last), "The hole={hole} is < last={last}");
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
    /// Returns it's new index
    #[inline(always)]
    fn _unsafe_sift_up(&mut self, index: usize) -> usize {
        debug_assert!(
            index < self.open.len(),
            "Node is way out of sync. Index out of bounds..."
        );
        debug_assert_eq!(
            self.search_tree[self.open[index].node_index].heap_index, index,
            "Node is out of sync."
        );

        // Can't improve
        if index == 0 {
            return index;
        }

        let mut pos = index;
        let mut parent = up(pos);
        while self.open[parent].rank > self.open[pos].rank {
            // Nodes are different and swapped. Swap the nodes to fix the order.
            self._unsafe_swap(parent, pos);
            debug_assert!(self.open[parent].rank < self.open[pos].rank);

            // Continue swapping upwards if needed..
            if parent == 0 {
                return parent;
            }
            pos = parent;
            parent = up(pos);
        }
        pos
    }

    // Swapping primitives
    /// Swaps two elements in the heap.
    ///
    /// For consistency in calling code `l < r` is checked.
    ///
    /// Keeps the intrusive indices in sync.
    #[inline(always)]
    fn _unsafe_swap(&mut self, l: usize, r: usize) {
        debug_assert!(l < r, "Swap({l}, {r}) uses wrong argument order");

        let len = self.open.len();
        debug_assert!(l < len, "Left  swap index {l} is OUT OF BOUNDS({len})");
        debug_assert!(r < len, "Right swap index {r} is OUT OF BOUNDS({len})");
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
    /// For consistency in calling code `l < r` is checked.
    ///
    /// Only keeps the index of the element going up in sync as we should shortly
    /// after remove the element that goes down.
    #[inline(always)]
    fn _unsafe_half_swap_down(&mut self, l: usize, r: usize) {
        debug_assert!(l < r, "HalfSwapDown({l}, {r}) is wrong");

        let len = self.open.len();
        debug_assert!(l < len, "Left  swap index {l} is OUT OF BOUNDS({len})");
        debug_assert!(r < len, "Right swap index {r} is OUT OF BOUNDS({len})");
        self.open.swap(l, r);
        self.search_tree[self.open[l].node_index].heap_index = l;
        debug_assert!(
            self.open[l].rank >= self.open[r].rank, // (Q: What if there's only one value? We still push node at the top down)
            "Half-assed swap down must be unfairly pushing a node down."
        );
        debug_assert!(
            self.search_tree[self.open[r].node_index].heap_index < r,
            "Node half-assed swapped down should still point to it's original index."
        );
    }

    pub fn write_memory_stats<W: std::io::Write>(&self, mut out: W) -> std::io::Result<()> {
        use size::Size;
        use std::mem::size_of;
        use thousands::Separable;

        writeln!(out, "DijkstraSearch Stats:")?;
        let s = size_of::<SearchTreeNode<St, A, C>>();
        let l = self.search_tree.len();
        writeln!(
            out,
            "  - |Nodes|:   {} ({})",
            l.separate_with_commas(),
            Size::from_bytes(l * s)
        )?;

        let s = size_of::<DijkstraHeapNode<C>>();
        let l = self.open.len();
        let c = self.open.capacity();
        writeln!(
            out,
            "  - |Open|:   {} ({})",
            l.separate_with_commas(),
            Size::from_bytes(l * s)
        )?;
        writeln!(
            out,
            "  - |Open|*:  {} ({})",
            c.separate_with_commas(),
            Size::from_bytes(c * s)
        )?;

        let s = size_of::<(St, SearchTreeIndex)>();
        let l = self.node_map.len();
        let c = self.node_map.capacity();
        writeln!(
            out,
            "  - |Index|:  {} ({})",
            l.separate_with_commas(),
            Size::from_bytes(l * s)
        )?;
        writeln!(
            out,
            "  - |Index|*: {} ({})",
            c.separate_with_commas(),
            Size::from_bytes(c * s)
        )?;

        let expanded_nodes = self.search_tree.len() - self.open.len();
        writeln!(
            out,
            "  - Expanded nodes: {}",
            expanded_nodes.separate_with_commas()
        )?;

        Ok(())
    }
    pub fn print_memory_stats(&self) {
        self.write_memory_stats(std::io::stdout().lock()).unwrap()
    }
}

impl<OP, Sp, St, A, C> Iterator for DijkstraSearch<OP, Sp, St, A, C>
where
    OP: ObjectiveProblem<Sp, St, A, C>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranking_maze2d() {
        use crate::problems::maze_2d::Maze2DCost;

        let c0: Maze2DCost = 0u32;
        let c1: Maze2DCost = 1u32;
        let c2: Maze2DCost = 2u32;

        assert!(DijkstraRank::new(c0) < DijkstraRank::new(c2));
        assert!(DijkstraRank::new(c2) == DijkstraRank::new(c2));
        assert!(DijkstraRank::new(c2) > DijkstraRank::new(c1));

        assert!(DijkstraRank::new(c0) < DijkstraRank::new(c1));
    }

    #[test]
    fn solve_maze2d() {
        use std::path::PathBuf;

        use crate::problem::BaseProblem;
        use crate::problems::maze_2d::Maze2DProblem;

        // Solve `two-paths.png` (`://data/problems/Maze2D/two-paths.png`)
        let problem =
            Maze2DProblem::try_from(PathBuf::from("data/problems/Maze2D/two-paths.png").as_path())
                .unwrap();
        let starts = problem.starts().to_vec();
        let goals = problem.goals().to_vec();
        let mut search = DijkstraSearch::new(problem);
        let path = search.find_next_goal().unwrap();

        assert!(starts.contains(&path.start.unwrap()));
        assert!(goals.contains(&path.end.unwrap()));
        assert_eq!(path.cost, 1805);
    }
}
