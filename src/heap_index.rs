use std::fmt::Debug;

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

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct HeapIndex {
    index: u32,
}
impl HeapIndex {
    #[inline(always)]
    pub fn zero() -> Self {
        Self { index: 0 }
    }
    #[inline(always)]
    pub fn next(&self) -> Self {
        Self {
            index: self.index + 1,
        }
    }
    #[inline(always)]
    pub fn from_usize(i: usize) -> Self {
        Self { index: i as u32 }
    }
    #[inline(always)]
    pub fn as_usize(&self) -> usize {
        self.index as usize
    }
    #[inline(always)]
    pub fn is_zero(&self) -> bool {
        self.index == 0
    }

    /// The parent
    ///
    /// ```
    /// use search::heap_index::HeapIndex;
    /// assert_eq!(HeapIndex::from_usize(1).up().as_usize(), 0);
    /// assert_eq!(HeapIndex::from_usize(2).up().as_usize(), 0);
    /// assert_eq!(HeapIndex::from_usize(3).up().as_usize(), 1);
    /// assert_eq!(HeapIndex::from_usize(4).up().as_usize(), 1);
    /// assert_eq!(HeapIndex::from_usize(5).up().as_usize(), 2);
    /// assert_eq!(HeapIndex::from_usize(6).up().as_usize(), 2);
    /// assert_eq!(HeapIndex::from_usize(25).up().as_usize(), 12);
    /// ```
    #[inline(always)]
    pub fn up(&self) -> Self {
        debug_assert!(self.index != 0);
        Self {
            index: (self.index - 1) >> 1,
        }
    }

    /// The left children
    ///
    /// ```
    /// use search::heap_index::HeapIndex;
    /// assert_eq!(HeapIndex::from_usize(0).up().as_usize(), 1);
    /// assert_eq!(HeapIndex::from_usize(1).up().as_usize(), 3);
    /// assert_eq!(HeapIndex::from_usize(3).up().as_usize(), 7);
    /// assert_eq!(HeapIndex::from_usize(11).up().as_usize(), 23);
    /// ```
    #[inline(always)]
    pub fn down_left(&self) -> Self {
        Self {
            index: (2 * self.index) + 1,
        }
    }

    /// The right children
    ///
    /// ```
    /// use search::heap_index::HeapIndex;
    /// assert_eq!(HeapIndex::from_usize(0).up().as_usize(), 2);
    /// assert_eq!(HeapIndex::from_usize(1).up().as_usize(), 4);
    /// assert_eq!(HeapIndex::from_usize(2).up().as_usize(), 6);
    /// assert_eq!(HeapIndex::from_usize(4).up().as_usize(), 10);
    /// assert_eq!(HeapIndex::from_usize(6).up().as_usize(), 14);
    /// ```
    #[inline(always)]
    pub fn down_right(&self) -> Self {
        Self {
            index: 2 * (self.index + 1),
        }
    }
}
