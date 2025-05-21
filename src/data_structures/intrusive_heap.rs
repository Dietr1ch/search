use core::intrinsics::unlikely;
use std::cmp::min;
use std::fmt::Debug;
use std::option::Option;

use crate::derank::derank;

type HeapIndex = usize;

/// Intrusive Heap Node
///
/// A struct that has an index to the Intrusive Heap.
pub trait IntrusiveHeapNode: Debug + Ord {
    fn set_heap_index(&mut self, i: HeapIndex);
    fn get_heap_index(&self) -> HeapIndex;
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

/// "Intrusive" Heap
///
/// NOTE: Intrusive data structures deal with mostly-opaque elements that carry
/// some data relevant to the data structure.
///
/// A Heap that embeds the vector index into the nodes so nodes can be found in
/// constant time.
///
/// So far this seems to make sense, but now how useful is this if you need to
/// "have" a node already? What we really want is to let the node live elsewhere
/// and allow finding the `IntrusiveHeapNode` corresponding to the "remote"
/// element quickly.
///
/// Now, how do we implement index updating after internal heap re-orderings? We
/// need to go to a "remote" node, and update it, but having a reference to it
/// would make the borrow checker haunt us forever.
/// If you do this manually, you use the index to a vector/arena as your
/// reference to the "remote" node, but how do we get that with a nice API?
/// It looks like something with a `IndexMut` implementation from this index
/// into the "remote" Node.
#[derive(Debug)]
pub struct IntrusiveHeap<N>
where
    N: IntrusiveHeapNode,
{
    heap: Vec<N>,
}

impl<N> IntrusiveHeap<N>
where
    N: IntrusiveHeapNode,
{
    pub fn new() -> Self {
        Self { heap: vec![] }
    }
    pub fn with_capacity(s: HeapIndex) -> Self {
        Self {
            heap: Vec::with_capacity(s),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn push(&mut self, n: N) -> HeapIndex {
        self.verify_heap();
        let heap_index = self.heap.len(); // Future heap_index

        self.heap.push(n);
        let heap_index = self._unsafe_sift_up(heap_index);

        self.verify_heap();
        heap_index
    }

    pub fn pop(&mut self) -> Option<N> {
        #[cfg(feature = "coz_profile")]
        coz::scope!("Pop");

				self.verify_heap();

				if unlikely(self.heap.len() <= 1) {
            return self.heap.pop();
				}

        Some(self._unsafe_pop_non_trivial_heap())
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
        for i in 0..self.heap.len() {
            // - Has the right intrusive index set.
            debug_assert_eq!(self.heap[i].get_heap_index(), i);

            // - Goes after its parent node, if any.
            if i == 0 {
                continue;
            }
            let p = up(i);
            debug_assert!(
                self.heap[p] <= self.heap[i],
                "Node[{p}]={:?} !<= child [{i}]={:?}. Out of heap of len={}",
                self.heap[p],
                self.heap[i],
                self.heap.len(),
            );
        }
    }

    // Implementation details

    /// Pops the top node from a Heap with at least 2 elements.
    ///
    /// Works by unfairly sifting down the top-node to the last level, where it can
    /// be swapped with the very last element of the array and popped
    /// Temporarily breaks invariants around the node sifting down unfairly.
    fn _unsafe_pop_non_trivial_heap(&mut self) -> N {
        #[cfg(feature = "coz_profile")]
        coz::scope!("PopNonTrivial");

        debug_assert!(!self.heap.is_empty(), "You can't pop from an empty heap");
        debug_assert!(
            self.heap.len() != 1,
            "It doesn't get easier. Why are you calling this?"
        );

        // Heap 101
        //
        //                            0                               : 1   2^d-1
        //              1                           2                 : 3
        //       3            4              5             6          : 7
        //   7      8      9     10      11     12     13     14      :15
        // 15 16  17 18  19 20  21 22  23  24  25 !   *  *   *  *     :31
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

        let len = self.heap.len();
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
            child += derank(&self.heap[child..min(child + HEAP_ARITY, len)]);

            debug_assert!(self.heap[hole] <= self.heap[child]);

            // Swap and update internal indices
            self._unsafe_half_swap_down(hole, child);

            // Update bubble-down indices
            hole = child;
            child = down_left(hole); // New left child
            if child >= self.heap.len() {
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

        let heap_node = self.heap.pop().unwrap();

        heap_node
    }

    /// Raises a node
    /// Returns it's new index
    #[inline(always)]
    fn _unsafe_sift_up(&mut self, index: usize) -> usize {
        debug_assert!(
            index < self.heap.len(),
            "Node is way out of sync. Index out of bounds..."
        );

        // Can't improve
        if index == 0 {
            return index;
        }

        let mut pos = index;
        let mut parent = up(pos);
        while self.heap[parent] > self.heap[pos] {
            // Nodes are different and swapped. Swap the nodes to fix the order.
            self._unsafe_swap(parent, pos);
            debug_assert!(self.heap[parent] < self.heap[pos]);

            // Continue swapping upwards if needed..
            if parent == 0 {
                return parent;
            }
            pos = parent;
            parent = up(pos);
        }
        pos
    }

    /// Lowers a node
    /// Returns it's new index
    #[inline(always)]
    fn _unsafe_sift_down(&mut self, mut index: usize) -> usize {
        let len = self.heap.len();
        debug_assert!(
            index < len,
            "Node is way out of sync. Index out of bounds..."
        );

        loop {
            // Find the best child
            let mut child = down_left(index);
            if child >= len {
                break;
            }

            debug_assert_eq!(child + HEAP_ARITY, down_right(index) + 1);
            child += derank(&self.heap[child..min(child + HEAP_ARITY, len)]);

            if self.heap[index] <= self.heap[child] {
                break;
            }

            self._unsafe_swap(index, child);
            debug_assert!(self.heap[index] <= self.heap[child]);

            index = child;
        }
        index
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

        let len = self.heap.len();
        debug_assert!(l < len, "Left  swap index {l} is OUT OF BOUNDS({len})");
        debug_assert!(r < len, "Right swap index {r} is OUT OF BOUNDS({len})");
        self.heap.swap(l, r);
        self.heap[l].set_heap_index(l);
        self.heap[r].set_heap_index(r);
        debug_assert!(
            self.heap[l] <= self.heap[r],
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

        let len = self.heap.len();
        debug_assert!(l < len, "Left  swap index {l} is OUT OF BOUNDS({len})");
        debug_assert!(r < len, "Right swap index {r} is OUT OF BOUNDS({len})");
        self.heap.swap(l, r);
        self.heap[l].set_heap_index(l);
        debug_assert!(
            self.heap[l] >= self.heap[r], // (=? What if there's only one value? We still push node at the top down)
            "Half-assed swap down must be unfairly pushing a node down."
        );
        self.heap[l].set_heap_index(l);
        debug_assert!(
            self.heap[r].get_heap_index() < r,
            "Node half-assed swapped down should still point to it's original index."
        );
    }
}

impl<N> Default for IntrusiveHeap<N>
where
    N: IntrusiveHeapNode,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<N> std::ops::Index<HeapIndex> for IntrusiveHeap<N>
where
    N: IntrusiveHeapNode,
{
    type Output = N;

    fn index(&self, index: HeapIndex) -> &Self::Output {
        &self.heap[index]
    }
}

impl<N> std::ops::IndexMut<HeapIndex> for IntrusiveHeap<N>
where
    N: IntrusiveHeapNode,
{
    fn index_mut(&mut self, index: HeapIndex) -> &mut Self::Output {
        &mut self.heap[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
    pub struct TestNode {
        name: String,
        heap_index: HeapIndex,
    }

    impl TestNode {
        pub fn new(name: String) -> Self {
            Self {
                name,
                heap_index: 0usize,
            }
        }
    }

    impl IntrusiveHeapNode for TestNode {
        fn set_heap_index(&mut self, i: HeapIndex) {
            self.heap_index = i;
        }
        fn get_heap_index(&self) -> HeapIndex {
            self.heap_index
        }
    }

    #[test]
    fn heap_works() {
        let mut heap = IntrusiveHeap::<TestNode>::new();

        let n = TestNode::new("aoeu".to_string());
        heap.push(n.clone());
        assert_eq!(heap.pop(), Some(n));
    }

    #[test]
    fn heap_sorts() {
        let mut heap = IntrusiveHeap::<TestNode>::new();

        assert_eq!(heap.push(TestNode::new("c".to_string())), 0usize);
        assert_eq!(heap.push(TestNode::new("e".to_string())), 1usize);
        assert_eq!(heap.push(TestNode::new("f".to_string())), 2usize);
        assert_eq!(heap.push(TestNode::new("a".to_string())), 0usize);
        assert_eq!(heap.push(TestNode::new("d".to_string())), 4usize);
        assert_eq!(heap.push(TestNode::new("b".to_string())), 5usize);

        assert_eq!(heap.pop().unwrap().name, "a");
        assert_eq!(heap.pop().unwrap().name, "b");
        assert_eq!(heap.pop().unwrap().name, "c");
        assert_eq!(heap.pop().unwrap().name, "d");
        assert_eq!(heap.pop().unwrap().name, "e");
        assert_eq!(heap.pop().unwrap().name, "f");
    }
}
