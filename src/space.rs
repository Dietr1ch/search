use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;

use itertools::Itertools;
use num_traits::SaturatingAdd;
use num_traits::sign::Unsigned;

pub trait Action: Copy + Clone + Debug + Display + PartialEq + Eq {}
pub trait State: Copy + Clone + Debug + Display + PartialEq + Eq + Hash {}
pub trait Cost:
    Copy
    + Clone
    + Debug
    + Display
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
    pub start: Option<S>,
    pub end: Option<S>,
    pub cost: C,
    pub actions: Vec<A>,
}

impl<S, A, C> Path<S, A, C>
where
    S: State,
    A: Action,
    C: Cost,
{
    #[inline(always)]
    #[must_use]
    pub(crate) fn new_from_start(start: S) -> Self {
        Self {
            start: Some(start),
            end: Some(start),
            cost: C::zero(),
            actions: vec![],
        }
    }

    #[inline(always)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    #[inline(always)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    /// Runs sanity checks
    // TODO: Verify path in a Space
    #[inline(always)]
    #[must_use]
    pub fn seems_valid(&self) -> bool {
        self.start.is_some() == self.end.is_some() && self.cost.valid()
    }

    #[inline(always)]
    pub(crate) fn append(&mut self, last_action: (S, A), c: C) {
        let (s, a) = last_action;
        self.actions.push(a);
        self.end = Some(s);
        self.cost = self.cost.saturating_add(&c);
    }

    /// Reverses the Path, likely making it invalid.
    ///
    /// Useful when naturally reconstructing paths in reverse.
    pub(crate) fn reverse(&mut self) {
        (self.end, self.start) = (self.start, self.end);
        self.actions.reverse();
    }

    #[inline(always)]
    #[must_use]
    pub fn new_empty() -> Self {
        Self {
            start: None,
            actions: vec![],
            end: None,
            cost: C::zero(),
        }
    }
}

impl<S, A, C> Display for Path<S, A, C>
where
    S: State,
    A: Action,
    C: Cost,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        debug_assert!(self.start.is_none() == self.end.is_none());

        match (self.start, self.end) {
            (Some(start), Some(end)) => {
                write!(
                    f,
                    "Path({}, {:?}:{}:{:?})",
                    self.cost,
                    start,
                    self.actions
                        .iter()
                        .take(20)
                        .map(|a| a.to_string())
                        .join(", "),
                    end
                )
            }
            (None, None) => write!(f, "Path()"),
            _ => unreachable!("Path::start and Path::end should both be Some or None"),
        }
    }
}

pub trait Space<St, A, C>: Clone + std::fmt::Debug
where
    St: State,
    A: Action,
    C: Cost,
{
    #[must_use]
    fn apply(&self, s: &St, a: &A) -> Option<St>;

    #[must_use]
    fn cost(&self, _s: &St, _a: &A) -> C;

    /// Expands a State
    // TODO: Check that (St, A) does not incurr in a lot of padding.
    #[must_use]
    fn neighbours(&self, s: &St) -> Vec<(St, A)>;
    /// Verify is a State is valid.
    #[must_use]
    fn valid(&self, s: &St) -> bool;

    #[must_use]
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
            *p == Path::<St, A, C>::new_empty()
        }
    }

    #[must_use]
    fn size(&self) -> Option<usize> {
        None
    }

    #[must_use]
    fn supports_random_state() -> bool {
        false
    }
    #[must_use]
    fn random_state<R: rand::Rng>(&self, _r: &mut R) -> Option<St> {
        debug_assert!(!Self::supports_random_state());
        None
    }
}

/// A space that allows computing paths in reverse.
pub trait ReversibleSpace<St, A, C>: Space<St, A, C> + Clone + std::fmt::Debug
where
    St: State,
    A: Action,
    C: Cost,
{
    /// Reverse action
    #[must_use]
    fn reverse(&self, a: &A) -> A;

    /// States that can reach a certain state.
    #[must_use]
    fn reverse_neighbours(&self, s: &St) -> Vec<(St, A)>;

    /// Some of the States that can reach a certain state.
    #[must_use]
    #[cfg(feature = "partial_reverse")]
    fn partial_reverse_neighbours(&self, s: &St) -> Vec<(St, A)>;
}

/// A general heuristic useful to move between any pair of states.
pub trait ObjectiveHeuristic<Sp, St, A, C>: std::fmt::Debug
where
    Sp: Space<St, A, C>,
    St: State,
    A: Action,
    C: Cost,
{
    #[must_use]
    fn h(_a: &St, _b: &St) -> C {
        C::zero()
    }
}

/// A more specific heuristic to move into a set of states satisfying some
/// particular condition.
pub trait ConditionHeuristic<Sp, St, A, C>: std::fmt::Debug
where
    Sp: Space<St, A, C>,
    St: State,
    A: Action,
    C: Cost,
{
    #[must_use]
    fn h(_s: &St) -> C {
        C::zero()
    }
}
