use crate::cost::Cost;
use crate::space::Action;
use crate::space::Space;
use crate::space::State;

/// The start of a problem, but lacking a goal.
pub trait BaseProblem<Sp, St, A, C>: std::fmt::Debug + Sized
where
    Sp: Space<St, A, C>,
    St: State,
    A: Action,
    C: Cost,
{
    #[must_use]
    fn space(&self) -> &Sp;
    #[must_use]
    fn starts(&self) -> &[St];
}

/// A problem where the goal is to reach some specific goal states.
///
/// NOTE: An heuristic here is a function `St -> St -> C`
/// NOTE: After finding a goal, the search fringe can be updated to direct the
/// search towards the remaining goals.
pub trait ObjectiveProblem<Sp, St, A, C>: BaseProblem<Sp, St, A, C>
where
    Sp: Space<St, A, C>,
    St: State,
    A: Action,
    C: Cost,
{
    /// The goal states.
    #[must_use]
    fn goals(&self) -> &[St];

    #[must_use]
    fn randomize<R: rand::Rng>(
        &mut self,
        r: &mut R,
        num_starts: u16,
        num_goals: u16,
    ) -> Option<Self>;
}

/// A problem where the goal is to reach states satisfying certain condition.
///
/// NOTE: An heuristic here is a function `St -> C`
/// NOTE: Finding multiple goals here tends to be more expensive as the search
/// cannot be steered away from goals already found.
pub trait ConditionProblem<Sp, St, A, C>: BaseProblem<Sp, St, A, C>
where
    Sp: Space<St, A, C>,
    St: State,
    A: Action,
    C: Cost,
{
    /// The goal condition.
    #[must_use]
    fn is_goal(&self, s: &St) -> bool;
}

/// A problem where there's Objective and Condition based goals.
///
/// NOTE: An heuristic here mixes both, objective and condition heuristics. If
/// the objectives match the goal condition the maximum of both heuristics can
/// be used, otherwise the minimum is needed to avoid letting the condition
/// heuristic to steer you away from objective states.
///
/// TODO: Think about this harder. Do we need to rank using both heuristics
/// separately?
pub trait MixedProblem<Sp, St, A, C>:
    ConditionProblem<Sp, St, A, C> + ObjectiveProblem<Sp, St, A, C>
where
    Sp: Space<St, A, C>,
    St: State,
    A: Action,
    C: Cost,
{
}
