use rustc_hash::FxHashSet;

use crate::space::Action;
use crate::space::Cost;
use crate::space::Space;
use crate::space::State;

pub trait Problem<Sp, St, A, C>: std::fmt::Debug + Sized
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

    fn randomize<R: rand::Rng>(
        &mut self,
        r: &mut R,
        num_starts: u16,
        num_goals: u16,
    ) -> Option<Self>;
}

/// An instance-specific heuristic.
pub trait ProblemHeuristic<P, Sp, St, A, C>: std::fmt::Debug
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
