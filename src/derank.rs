// A graph-network function to find arg-min
//
// This naturally lends to a recursive problem, but in practice we eventually
// want to run the tournament left-to-right once we hit the cache-line size, so
// it'll look more like a fold with the best candidate, and the tournament slice
// that maximices IPC. I haven't digged into this and it gets tricky if the
// comparison function isn't trivial as the CPU may run better on smaller
// tournaments than what could be fetched.

/// Core comparison and index selection
#[inline(always)]
#[must_use]
fn fight<T: PartialOrd>(a: &[T], l: usize, r: usize) -> usize {
    if a[l] <= a[r] { l } else { r }
}

// 0   1
// *   *
//  \ /
//   *

#[inline(always)]
#[must_use]
pub fn derank_2<T: PartialOrd>(a: &[T]) -> usize {
    debug_assert!(a.len() == 2);
    // fight(
    //     a, //
    //     derank_1(a[0..1]),
    //     derank_1(a[1..2]) + 1,
    // )
    fight(a, 0, 1)
}

// 0   1   2
// *   *   *
//  \ /    |
//   *     *
//    \   /
//      *
#[inline(always)]
#[must_use]
pub fn derank_3<T: PartialOrd>(a: &[T]) -> usize {
    debug_assert!(a.len() == 3);
    // fight(
    //     a, //
    //     derank_2(a[0..2]),
    //     2,
    // )
    fight(
        a,              //
        fight(a, 0, 1), //
        2,
    )
}

// 0   1   2   3
// *   *   *   *
//  \ /     \ /
//   *       *
//    \     /
//       *
#[inline(always)]
#[must_use]
pub fn derank_4<T: PartialOrd>(a: &[T]) -> usize {
    debug_assert!(a.len() == 4);
    // fight(
    //     a, //
    //     derank_2(a[0..2]),
    //     derank_2(a[2..4]) + 2,
    // )
    fight(
        a, //
        fight(a, 0, 1),
        fight(a, 2, 3),
    )
}

// 0   1 2   3   4
// *   * *   *   *
//  \ /   \ /   /
//   *     *   /
//    \   /   /
//      *    /
//       \  /
//         *
#[inline(always)]
#[must_use]
pub fn derank_5<T: PartialOrd>(a: &[T]) -> usize {
    debug_assert!(a.len() == 5);
    // fight(
    //     a, //
    //     derank_4(a[0..4]),
    //     4,
    // )
    fight(
        a,
        fight(
            a, //
            fight(a, 0, 1),
            fight(a, 2, 3),
        ),
        4,
    )
}

// 0   1 2   3 4   5
// *   * *   * *   *
//  \ /   \ /   \ /
//   *     *     *
//    \   /     /
//      *      /
//       \    /
//         *
#[inline(always)]
#[must_use]
pub fn derank_6<T: PartialOrd>(a: &[T]) -> usize {
    debug_assert!(a.len() == 6);
    // fight(
    //     a, //
    //     derank_4(a[0..4]),
    //     derank_2(a[4..6]) + 4,
    // )
    fight(
        a,
        fight(
            a, //
            fight(a, 0, 1),
            fight(a, 2, 3),
        ),
        fight(a, 4, 5),
    )
}

// 0   1 2   3 4   5   6
// *   * *   * *   *   *
//  \ /   \ /   \ /   /
//   *     *     *   /
//    \   /       \ /
//      *          *
//        \       /
//            *
#[inline(always)]
#[must_use]
pub fn derank_7<T: PartialOrd>(a: &[T]) -> usize {
    debug_assert!(a.len() == 7);
    // fight(
    //     a, //
    //     derank_4(a[0..4]),
    //     derank_3(a[4..7]) + 4,
    // )
    fight(
        a,
        fight(
            a, //
            fight(a, 0, 1),
            fight(a, 2, 3),
        ),
        fight(
            a,
            fight(a, 4, 5), //
            6,
        ),
    )
}

// 0   1 2   3 4   5 6   7
// *   * *   * *   * *   *
//  \ /   \ /   \ /   \ /
//   *     *     *     *
//    \   /       \   /
//      *           *
//        \        /
//            *
#[inline(always)]
#[must_use]
pub fn derank_8<T: PartialOrd>(a: &[T]) -> usize {
    debug_assert!(a.len() == 8);
    // fight(
    //     a, //
    //     derank_4(a[0..4]),
    //     derank_4(a[4..8]) + 4,
    // )
    fight(
        a,
        fight(
            a, //
            fight(a, 0, 1),
            fight(a, 2, 3),
        ),
        fight(
            a, //
            fight(a, 4, 5),
            fight(a, 6, 7),
        ),
    )
}

// 0   1 2   3 4   5 6   7 8   9 10 11 12 13 14  15
// *   * *   * *   * *   * *   * *   * *   * *   *
//  \ /   \ /   \ /   \ /   \ /   \ /   \ /   \ /
//   *     *     *     *     *     *     *     *
//    \   /       \   /       \   /       \   /
//      *           *           *           *
//        \        /              \        /
//            *                       *
//              \                    /
//                        *
#[inline(always)]
#[must_use]
fn derank_16<T: PartialOrd>(a: &[T]) -> usize {
    debug_assert!(a.len() == 16);
    // fight(
    //     a, //
    //     derank_8(a[0..8]),
    //     derank_8(a[8..16]) + 8,
    // )
    fight(
        a,
        fight(
            a,
            fight(
                a, //
                fight(a, 0, 1),
                fight(a, 2, 3),
            ),
            fight(
                a, //
                fight(a, 4, 5),
                fight(a, 6, 7),
            ),
        ),
        fight(
            a,
            fight(
                a, //
                fight(a, 8, 9),
                fight(a, 10, 11),
            ),
            fight(
                a, //
                fight(a, 12, 13),
                fight(a, 14, 15),
            ),
        ),
    )
}

#[inline(always)]
#[must_use]
pub fn derank<T: PartialOrd>(a: &[T]) -> usize {
    match a.len() {
        1 => 0usize,
        2 => derank_2(a),
        3 => derank_3(a),
        4 => derank_4(a),
        5 => derank_5(a),
        6 => derank_6(a),
        7 => derank_7(a),
        8 => derank_8(a),
        16 => derank_16(a),
        _ => unreachable!(),
    }
}

pub fn linear_min_index<T: PartialOrd>(xs: &[T]) -> usize {
    assert!(!xs.is_empty());

    let mut min_i = 0;
    for (i, x) in xs.iter().enumerate() {
        if *x < xs[min_i] {
            min_i = i;
        }
    }
    min_i
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_02() {
        let a = vec![0u8, 1u8];
        assert_eq!(derank(&a), linear_min_index(&a));
    }

    #[test]
    fn verify_03() {
        let a = vec![0u8, 1u8, 2u8];
        assert_eq!(derank(&a), linear_min_index(&a));
    }

    #[test]
    fn verify_04() {
        let a = vec![0u8, 1u8, 3u8, 2u8];
        assert_eq!(derank(&a), linear_min_index(&a));
    }

    #[test]
    fn verify_05() {
        let a = vec![1u8, 0u8, 4u8, 3u8, 2u8];
        assert_eq!(derank(&a), linear_min_index(&a));
    }

    #[test]
    fn verify_06() {
        let a = vec![1u8, 5u8, 0u8, 4u8, 3u8, 2u8];
        assert_eq!(derank(&a), linear_min_index(&a));
    }

    #[test]
    fn verify_07() {
        let a = vec![1u8, 5u8, 0u8, 4u8, 6u8, 3u8, 2u8];
        assert_eq!(derank(&a), linear_min_index(&a));
    }

    #[test]
    fn verify_08() {
        let a = vec![1u8, 5u8, 0u8, 4u8, 6u8, 3u8, 7u8, 2u8];
        assert_eq!(derank(&a), linear_min_index(&a));
    }

    #[test]
    fn verify_16() {
        let a = vec![
            1u8, 5u8, 0u8, 5u8, 0u8, 4u8, 4u8, 6u8, 3u8, 7u8, 2u8, 1u8, 6u8, 3u8, 7u8, 2u8,
        ];
        assert_eq!(derank(&a), linear_min_index(&a));
    }
}
