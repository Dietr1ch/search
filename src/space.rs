use std::fmt::Debug;
use std::hash::Hash;

use num_traits::sign::Unsigned;
use num_traits::SaturatingAdd;

pub trait Action: Copy + Clone + Debug + PartialEq + Eq {}
pub trait State: Copy + Clone + Debug + PartialEq + Eq + Hash {}
pub trait Cost:
    Copy
    + Clone
    + Debug
    + std::fmt::Display
    + PartialEq
    + Eq
    + PartialOrd
    + Ord
    + SaturatingAdd
    + Unsigned
    + num_traits::bounds::UpperBounded
    + std::ops::Add
    + std::ops::AddAssign
{
    fn valid(&self) -> bool {
        *self != Self::max_value()
    }
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "inspect", derive(Clone))]
pub struct Path<S, A, C>
where
    S: State,
    A: Action,
    C: Cost,
{
    start: Option<S>,
    end: Option<S>,
    cost: C,
    actions: Vec<A>,
}

impl<S, A, C> Path<S, A, C>
where
    S: State,
    A: Action,
    C: Cost,
{
    #[inline(always)]
    pub fn new_from_start(start: S) -> Self {
        Self {
            start: Some(start),
            end: Some(start),
            cost: C::zero(),
            actions: vec![],
        }
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    /// Runs sanity checks
    #[inline(always)]
    pub fn seems_valid(&self) -> bool {
        self.start.is_some() == self.end.is_some() && self.cost.valid()
    }

    #[inline(always)]
    pub fn append(&mut self, last_action: (S, A), c: C) {
        let (s, a) = last_action;
        self.actions.push(a);
        self.end = Some(s);
        self.cost = self.cost.saturating_add(&c);
    }

    /// Reverses the Path, likely making it invalid.
    ///
    /// Useful when naturally reconstructing paths in reverse.
    pub fn reverse(&mut self) {
        (self.end, self.start) = (self.start, self.end);
        self.actions.reverse();
    }

    #[inline(always)]
    pub fn empty() -> Self {
        Self {
            start: None,
            actions: vec![],
            end: None,
            cost: C::zero(),
        }
    }
}

impl<S, A, C> std::fmt::Display for Path<S, A, C>
where
    S: State,
    A: Action,
    C: Cost,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        debug_assert!(self.start.is_none() == self.end.is_none());

        match (self.start, self.end) {
            (Some(start), Some(end)) => {
                writeln!(
                    f,
                    "Path({}, {:?}:{:?}:{:?}):",
                    self.cost, start, self.actions, end
                )
            }
            (None, None) => writeln!(f, "Path()"),
            _ => unreachable!("Path::start and Path::end should both be Some or None"),
        }
    }
}

pub trait Space<St, A, C>: std::fmt::Debug
where
    St: State,
    A: Action,
    C: Cost,
{
    fn apply(&self, s: &St, a: &A) -> Option<St>;

    fn cost(&self, _s: &St, _a: &A) -> C {
        C::one()
    }
    fn neighbours(&self, s: &St) -> Vec<(St, A)>;
    fn valid(&self, s: &St) -> bool;

    fn valid_path(&self, p: &Path<St, A, C>) -> bool {
        if let Some(start) = p.start {
            // Verify path
            let mut state: St = start;
            for a in &p.actions {
                match self.apply(&state, a) {
                    Some(new_state) => state = new_state,
                    None => return false,
                }
            }
            if let Some(end) = p.end {
                return end == state;
            }
            false
        } else {
            // Empty paths are fine
            *p == Path::<St, A, C>::empty()
        }
    }

    fn size(&self) -> Option<usize> {
        None
    }
}

use rustc_hash::FxHashSet;

pub trait Problem<Sp, St, A, C>: std::fmt::Debug
where
    Sp: Space<St, A, C>,
    St: State,
    A: Action,
    C: Cost,
{
    fn space(&self) -> &Sp;
    fn starts(&self) -> &Vec<St>;
    fn goals(&self) -> &FxHashSet<St>;

    fn is_goal(&self, s: &St) -> bool {
        self.goals().contains(s)
    }
}
