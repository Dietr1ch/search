// Heap intrinsic operations implemented externally.
//
// A heap is a tree-like structure where every subtree's root has a better score
// than all the other nodes in the subtree.
//
// This is often implemented with an array that's traversed in a non-linear way.
// These are the indices we assign to each node.
//
// ```text
//                           0
//              1                         2
//       3            4            5             6
//   7      8      9     10    11     12     13     14
// 15 16  17 18  19 20  21 22 23 24  25
// ```
//
// The last level will often be incomplete
//
// You can easily go up, down-left, and down-right from any index with,
//   - Up:         `(i-1)//2`
//   - Down-left:  `(2*i) + 1`
//   - Down-right: `2(i+1)`

/// The parent node
///
/// ```
/// use search::heap_primitives::index_parent;
/// assert_eq!(index_parent::<2>(1), 0);
/// assert_eq!(index_parent::<2>(2), 0);
/// assert_eq!(index_parent::<2>(3), 1);
/// assert_eq!(index_parent::<2>(4), 1);
/// assert_eq!(index_parent::<2>(5), 2);
/// assert_eq!(index_parent::<2>(6), 2);
/// assert_eq!(index_parent::<2>(25), 12);
/// ```
#[inline(always)]
#[must_use]
pub fn index_parent<const A: usize>(i: usize) -> usize {
    (i - 1) / A
}

/// The left children
///
/// ```
/// use search::heap_primitives::index_first_children;
/// assert_eq!(index_first_children::<2usize>(0), 1);
/// assert_eq!(index_first_children::<2usize>(1), 3);
/// assert_eq!(index_first_children::<2usize>(3), 7);
/// assert_eq!(index_first_children::<2usize>(11), 23);
/// ```
#[inline(always)]
#[must_use]
pub fn index_first_children<const A: usize>(i: usize) -> usize {
    (A * i) + 1
}

/// The last children
///
/// ```
/// use search::heap_primitives::index_last_children;
/// assert_eq!(index_last_children::<2usize>(0), 2);
/// assert_eq!(index_last_children::<2usize>(1), 4);
/// assert_eq!(index_last_children::<2usize>(2), 6);
/// assert_eq!(index_last_children::<2usize>(6), 14);
/// assert_eq!(index_last_children::<2usize>(4), 10);
/// ```
#[inline(always)]
#[must_use]
pub fn index_last_children<const A: usize>(i: usize) -> usize {
    A * (i + 1)
}

pub fn size_of_cacheline_arity<T: Sized>() -> usize {
    let s = std::mem::size_of::<T>();
    std::cmp::max(128 / s, 2usize)
}
