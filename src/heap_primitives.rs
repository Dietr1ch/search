// Heap intrinsic operations implemented externally.
//
// A heap is a tree-like structure where every subtree's root has a better score
// than all the other nodes in the subtree.
//
// This is often implemented with an array that's traversed in a non-linear way.
// These are the indices we assign to each node.
//
//                           0
//              1                         2
//       3            4            5             6
//   7      8      9     10    11     12     13     14
// 15 16  17 18  19 20  21 22 23 24  25
//
// The last level will often be incomplete
//
// You can easily go up, down-left, and down-right from any index with,
//   - Up: (i-1)//2
//   - DL: (2*i) + 1
//   - DR: 2(i+1)

/// The parent
///
/// ```
/// use search::heap_primitives::index_up;
/// assert_eq!(index_up(1), 0);
/// assert_eq!(index_up(2), 0);
/// assert_eq!(index_up(3), 1);
/// assert_eq!(index_up(4), 1);
/// assert_eq!(index_up(5), 2);
/// assert_eq!(index_up(6), 2);
/// assert_eq!(index_up(25), 12);
/// ```
#[inline(always)]
pub fn index_up(i: usize) -> usize {
    (i - 1) >> 1
    // TODO: Introduce arity as a parameter
}

/// The left children
///
/// ```
/// use search::heap_primitives::index_down_left;
/// assert_eq!(index_down_left(0), 1);
/// assert_eq!(index_down_left(1), 3);
/// assert_eq!(index_down_left(3), 7);
/// assert_eq!(index_down_left(11), 23);
/// ```
#[inline(always)]
pub fn index_down_left(i: usize) -> usize {
    // TODO: Introduce arity as a parameter
    (2 * i) + 1
}

/// The right children
///
/// ```
/// use search::heap_primitives::index_down_right;
/// assert_eq!(index_down_right(0), 2);
/// assert_eq!(index_down_right(1), 4);
/// assert_eq!(index_down_right(2), 6);
/// assert_eq!(index_down_right(6), 14);
/// assert_eq!(index_down_right(4), 10);
/// ```
#[inline(always)]
pub fn index_down_right(i: usize) -> usize {
    // TODO: Introduce arity as a parameter
    2 * (i + 1)
}
