use std::fmt::Debug;
use std::ptr::NonNull;

use rustc_hash::FxHashMap;

use crate::heap_primitives::index_down_left;
use crate::heap_primitives::index_up;
use crate::space::Action;
use crate::space::Cost;
use crate::space::Path;
use crate::space::Problem;
use crate::space::Space;
use crate::space::State;

pub trait Heuristic<P, Sp, St, A, C>
where
    P: Problem<Sp, St, A, C>,
    Sp: Space<St, A, C>,
    St: State,
    A: Action,
    C: Cost,
{
    fn h(_p: &P, _s: &St) -> C {
        C::zero()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AStarRank<C: Cost> {
    r: (C, C),
}
impl<C> AStarRank<C>
where
    C: Cost,
{
    pub fn new(g: C, h: C) -> Self {
        Self {
            r: (g.saturating_add(&h), h),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "inspect", derive(Clone))]
pub struct AStarNode<St, A, C>
where
    St: State,
    A: Action,
    C: Cost,
{
    pub parent: Option<(NonNull<Self>, A)>,
    pub state: St,
    pub g: C,
    pub h: C,
}

impl<St, A, C> AStarNode<St, A, C>
where
    St: State,
    A: Action,
    C: Cost,
{
    pub fn new(s: St, g: C, h: C) -> Self {
        Self {
            parent: None,
            state: s,
            g,
            h,
        }
    }
    pub fn new_from_parent(s: St, parent: (NonNull<Self>, A), g: C, h: C) -> Self {
        Self {
            parent: Some(parent),
            state: s,
            g,
            h,
        }
    }

    pub fn reach(&mut self, new_parent: (NonNull<Self>, A), g: C, h: C) {
        debug_assert!(g < self.g);
        self.parent = Some(new_parent);
        self.g = g;
        self.h = h;
    }
}

impl<St, A, C> AStarNode<St, A, C>
where
    St: State,
    A: Action,
    C: Cost,
{
    pub fn state(&self) -> &St {
        &self.state
    }
    pub fn rank(&self) -> AStarRank<C> {
        AStarRank::new(self.g, self.h)
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "inspect", derive(Clone))]
struct AStarHeapNode<C>
where
    C: Cost,
{
    rank: AStarRank<C>,
    index: usize,
}

use std::marker::PhantomData;

#[derive(Debug)]
#[cfg_attr(feature = "inspect", derive(Clone))]
pub struct AStarSearch<P, H, Sp, St, A, C>
where
    P: Problem<Sp, St, A, C>,
    H: Heuristic<P, Sp, St, A, C>,
    Sp: Space<St, A, C>,
    St: State,
    A: Action,
    C: Cost,
{
    // TODO: Clean PhantomData
    phantom_heuristic: PhantomData<H>,
    phantom_space: PhantomData<Sp>,
    phantom_action: PhantomData<A>,

    nodes: Vec<AStarNode<St, A, C>>,
    open: Vec<AStarHeapNode<C>>,
    /// Amalgamation of,
    /// - The `HashMap<St, &Node>`, but using just the node index
    /// - The "Closed Set" `HashSet<St>`
    node_index: FxHashMap<St, (usize, bool)>,

    problem: P,
}

type Idx = usize;

impl<P, H, Sp, St, A, C> AStarSearch<P, H, Sp, St, A, C>
where
    P: Problem<Sp, St, A, C>,
    H: Heuristic<P, Sp, St, A, C>,
    Sp: Space<St, A, C>,
    St: State,
    A: Action,
    C: Cost,
{
    pub fn new(p: P) -> Self {
        let mut search = Self {
            // TODO: Clean PhantomData
            phantom_heuristic: PhantomData,
            phantom_space: PhantomData,
            phantom_action: PhantomData,

            nodes: vec![],
            open: vec![],
            node_index: FxHashMap::default(),

            problem: p,
        };

        for s in search.problem.starts() {
            let g = C::zero();
            let h: C = H::h(&search.problem, s);
            let node = AStarNode::<St, A, C>::new(*s, g, h);

            // search.push(node);
            let i = search.nodes.len();
            search.open.push(AStarHeapNode {
                rank: node.rank(),
                index: i,
            });
            search.node_index.insert(*s, (i, false));
            search.nodes.push(node);
        }

        search
    }

    fn build_path(&self, node: &AStarNode<St, A, C>) -> Path<St, A, C> {
        println!("Building A* path...");
        let mut p = Path::<St, A, C>::new_from_start(*node.state());

        let mut node: NonNull<AStarNode<St, A, C>> = NonNull::from_ref(node);
        while let Some((parent_node, a)) = unsafe { node.as_ref() }.parent {
            let state = unsafe { *parent_node.as_ref().state() };
            let c: C = self.problem.space().cost(&state, &a);
            debug_assert!(c > C::zero());
            p.append((state, a), c);
            node = parent_node;
        }

        p.reverse();
        p
    }

    pub fn find_first(&mut self) -> Option<Path<St, A, C>> {
        while let Some(node) = self.pop() {
            let node_ptr = NonNull::from_ref(&self.nodes[node]);
            let node: &AStarNode<St, A, C> = unsafe { node_ptr.as_ref() };

            let g = node.g;
            let state: &St = node.state();
            debug_assert!(!self.is_closed(state));
            println!("Popped {:?}", node);

            if self.problem.is_goal(state) {
                return Some(self.build_path(node));
            }

            // Mark as closed
            self.mark_closed(state);
            // Expand state
            for (s, a) in self.problem.space().neighbours(state) {
                let c: C = self.problem.space().cost(&s, &a);

                println!("  Found: {s:?}:{a:?}");
                if self.is_closed(&s) {
                    // TODO: Assert new cost isn't better.
                    continue;
                }
                let new_g = g + c;
                // TODO: Check if node is already in open
                // if new_g >= g {
                //     continue;
                // }

                let new_h = H::h(&self.problem, &s);
                self.push(AStarNode::new_from_parent(s, (node_ptr, a), new_g, new_h));
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
    pub fn pop_node(&mut self) -> Option<&AStarNode<St, A, C>> {
        match self.pop() {
            Some(i) => Some(&self.nodes[i]),
            None => None,
        }
    }

    #[inline(always)]
    fn pop(&mut self) -> Option<Idx> {
        match self.open.len() {
            0 | 1 => self.open.pop().map(|n| n.index),
            _ => {
                self.verify_heap();
                let node = self._unsafe_pop_non_trivial_heap();
                self.verify_heap();
                Some(node)
            }
        }
    }

    fn push(&mut self, node: AStarNode<St, A, C>) {
        let i = self.nodes.len();
        self.open.push(AStarHeapNode {
            rank: node.rank(),
            index: i,
        });
        self.node_index.insert(*node.state(), (i, false));
        self.nodes.push(node);
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
            debug_assert!(e.index == i);

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

        let node = self.open.pop().unwrap();
        debug_assert_eq!(
            node.index, 0,
            "Top node half-assed swapped down should still have it's 0 index"
        );

        node.index
    }

    /// Raises a node
    #[inline(always)]
    fn _unsafe_sift_up(&mut self, index: usize) -> usize {
        debug_assert!(
            index < self.open.len(),
            "Node is way out of sync. Index out of bounds..."
        );
        debug_assert!(self.open[index].index == index, "Node is out of sync.");

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
        self.open[l].index = l;
        self.open[r].index = r;
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
        self.open[l].index = l;
        debug_assert!(
            self.open[l].rank >= self.open[r].rank, // (=? What if there's only one value? We still push node at the top down)
            "Half-assed swap down must be unfairly pushing a node down."
        );
        debug_assert!(
            self.open[r].index < r,
            "Node half-assed swapped down should still point to it's original index."
        );
    }
}
