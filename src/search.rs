use std::fmt::Debug;

use nonmax::NonMaxUsize;
use rustc_hash::FxHashMap;

use crate::heap_primitives::index_down_left;
use crate::heap_primitives::index_up;
use crate::space::Action;
use crate::space::Cost;
use crate::space::Path;
use crate::space::Problem;
use crate::space::Space;
use crate::space::State;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DijkstraRank<C: Cost> {
    r: C,
}
impl<C> DijkstraRank<C>
where
    C: Cost,
{
    pub fn new(g: C) -> Self {
        Self { r: g }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "inspect", derive(Clone))]
pub struct DijkstraNode<St, A, C>
where
    St: State,
    A: Action,
    C: Cost,
{
    pub parent: Option<(NonMaxUsize, A)>,
    pub state: St,
    pub g: C,
    heap_index: usize,
}

impl<St, A, C> DijkstraNode<St, A, C>
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

impl<St, A, C> DijkstraNode<St, A, C>
where
    St: State,
    A: Action,
    C: Cost,
{
    pub fn state(&self) -> &St {
        &self.state
    }
    pub fn rank(&self) -> DijkstraRank<C> {
        DijkstraRank::new(self.g)
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "inspect", derive(Clone))]
struct DijkstraHeapNode<C>
where
    C: Cost,
{
    rank: DijkstraRank<C>,
    /// The index of this node in the Node Arena
    node_index: usize,
}

use std::marker::PhantomData;

#[derive(Debug)]
#[cfg_attr(feature = "inspect", derive(Clone))]
pub struct DijkstraSearch<P, Sp, St, A, C>
where
    P: Problem<Sp, St, A, C>,
    Sp: Space<St, A, C>,
    St: State,
    A: Action,
    C: Cost,
{
    nodes: Vec<DijkstraNode<St, A, C>>,
    open: Vec<DijkstraHeapNode<C>>,
    /// Amalgamation of,
    /// - The `HashMap<St, &Node>`, but using just the node index
    /// - The "Closed Set" `HashSet<St>`
    node_index: FxHashMap<St, (usize, bool)>,

    problem: P,

    // TODO: Clean PhantomData
    _phantom_space: PhantomData<Sp>,
    _phantom_action: PhantomData<A>,
}

type Idx = usize;

impl<P, Sp, St, A, C> DijkstraSearch<P, Sp, St, A, C>
where
    P: Problem<Sp, St, A, C>,
    Sp: Space<St, A, C>,
    St: State,
    A: Action,
    C: Cost,
{
    pub fn new(p: P) -> Self {
        let mut search = Self {
            nodes: vec![],
            open: vec![],
            node_index: FxHashMap::default(),

            problem: p,

            // TODO: Clean PhantomData
            _phantom_space: PhantomData,
            _phantom_action: PhantomData,
        };

        let starts = search.problem.starts().clone();
        for s in starts {
            let node_index = search.nodes.len();
            let heap_index = search.open.len();

            let g = C::zero();
            let node = DijkstraNode::<St, A, C>::new(heap_index, s, g);

            // search.push(node);
            search.open.push(DijkstraHeapNode {
                rank: node.rank(),
                node_index,
            });
            search.node_index.insert(s, (node_index, false));
            search.nodes.push(node);
            search._unsafe_sift_up(heap_index);

            search.verify_heap();
        }

        search
    }

    fn build_path(&self, mut node_index: usize) -> Path<St, A, C> {
        let e = &self.nodes[node_index];
        let mut path = Path::<St, A, C>::new_from_start(*e.state());

        while let Some((parent_index, a)) = self.nodes[node_index].parent {
            let p = &self.nodes[parent_index.get()];
            let s = p.state();
            let c: C = self.problem.space().cost(s, &a);
            debug_assert!(c != C::zero());

            path.append((*s, a), c);
            debug_assert!(node_index != parent_index.get());
            node_index = parent_index.get();
        }

        path.reverse();
        path
    }

    pub fn find_first(&mut self) -> Option<Path<St, A, C>> {
        while let Some(node_index) = self.pop() {
            let state = *self.nodes[node_index].state();
            let g: C = self.nodes[node_index].g;
            debug_assert!(!self.is_closed(&state));

            if self.problem.is_goal(&state) {
                return Some(self.build_path(node_index));
            }

            // Mark as closed
            self.mark_closed(&state);
            // Expand state
            for (s, a) in self.problem.space().neighbours(&state) {
                let c: C = self.problem.space().cost(&s, &a);

                match self.node_index.get(&s) {
                    Some((_, true)) => {
                        // Closed
                        continue;
                    }
                    Some((node_index, false)) => {
                        // Existing, but unexplored node
                        let neigh = &mut self.nodes[*node_index];
                        let neigh_heap_index = neigh.heap_index;
                        let new_g = g + c;
                        if new_g < neigh.g {
                            // Found better path to existing node
                            neigh.g = new_g;
                            self.open[neigh_heap_index].rank = neigh.rank();
                            self._unsafe_sift_up(neigh_heap_index);
                        }
                    }
                    None => {
                        // New node
                        let new_g = g + c;
                        let new_heap_index = self.open.len();
                        self.push(DijkstraNode::new_from_parent(
                            new_heap_index,
                            s,
                            (NonMaxUsize::new(node_index).unwrap(), a),
                            new_g,
                        ));
                    }
                }
            }
        }

        None
    }

    // pub fn find_all(&mut self) -> Vec<Path<St, A>> {
    //     while let Some(node) = self.pop() {
    //         println!("Popped {}", node);
    //     }

    //     self.push();

    //     None
    // }

    #[inline(always)]
    pub fn find_node(&self, s: &St) -> Option<Idx> {
        self.node_index.get(s).map(|(i, _is_closed)| *i)
    }
    #[inline(always)]
    pub fn is_closed(&self, s: &St) -> bool {
        match self.node_index.get(s) {
            Some((_index, is_closed)) => *is_closed,
            None => false,
        }
    }
    #[inline(always)]
    fn mark_closed(&mut self, s: &St) {
        match self.node_index.get(s) {
            Some((_index, true)) => {
                // Closed a closed state
            }
            Some((index, false)) => {
                self.node_index.insert(*s, (*index, true));
            }
            None => {
                unreachable!("Tried closing a state without a node");
            }
        }
    }

    #[inline(always)]
    pub fn pop_node(&mut self) -> Option<&DijkstraNode<St, A, C>> {
        match self.pop() {
            Some(i) => Some(&self.nodes[i]),
            None => None,
        }
    }

    #[inline(always)]
    fn pop(&mut self) -> Option<Idx> {
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

    fn push(&mut self, mut node: DijkstraNode<St, A, C>) {
        self.verify_heap();
        debug_assert!(!self.is_closed(&node.state));

        let node_index = self.nodes.len();
        let heap_index = self.open.len();

        self.open.push(DijkstraHeapNode {
            rank: node.rank(),
            node_index,
        });
        self.node_index.insert(*node.state(), (node_index, false));
        node.heap_index = heap_index;
        self.nodes.push(node);
        self._unsafe_sift_up(heap_index);

        self.verify_heap();
    }

    #[inline(always)]
    #[cfg(not(feature = "inspect"))]
    pub fn verify_heap(&self) {
        // All good... (hopefully)
    }
    #[inline(always)]
    #[cfg(feature = "inspect")]
    pub fn verify_heap(&self) {
        // Every node,
        for (i, e) in self.open.iter().enumerate() {
            // - Has the right intrusive index set.
            debug_assert!(self.nodes[e.node_index].heap_index == i);

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
    fn _unsafe_pop_non_trivial_heap(&mut self) -> usize {
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
            self.nodes[heap_node.node_index].heap_index, 0,
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
            self.nodes[self.open[index].node_index].heap_index == index,
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
        self.nodes[self.open[l].node_index].heap_index = l;
        self.nodes[self.open[r].node_index].heap_index = r;
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
        self.nodes[self.open[l].node_index].heap_index = l;
        debug_assert!(
            self.open[l].rank >= self.open[r].rank, // (=? What if there's only one value? We still push node at the top down)
            "Half-assed swap down must be unfairly pushing a node down."
        );
        debug_assert!(
            self.nodes[self.open[r].node_index].heap_index < r,
            "Node half-assed swapped down should still point to it's original index."
        );
    }
}
