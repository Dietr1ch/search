use std::cmp::Eq;
use std::fmt::Debug;

use derive_more::Display;
use num_traits::One;
use num_traits::SaturatingAdd;
use num_traits::Zero;
use num_traits::bounds::UpperBounded;
use ordered_float::FloatCore;
use ordered_float::OrderedFloat;

use crate::cost::Cost;

#[derive(Copy, Clone, Default, Debug, Display)]
#[repr(transparent)]
#[display("${_0}")]
pub struct FloatCost<F: FloatCore>(pub OrderedFloat<F>);

impl<F> Cost for FloatCost<F>
where
    FloatCost<F>: Debug + std::ops::AddAssign + Ord + Eq + UpperBounded,
    F: FloatCore + std::fmt::Display,
{
}

impl<F> FloatCost<F>
where
    F: FloatCore,
{
    pub fn new(f: F) -> Self {
        Self(OrderedFloat(f))
    }
    pub fn from_ordered_float(f: OrderedFloat<F>) -> Self {
        Self(f)
    }

    #[inline(always)]
    pub fn infinity() -> Self {
        Self(OrderedFloat::infinity())
    }
}

impl<F> std::ops::Add for FloatCost<F>
where
    OrderedFloat<F>: std::ops::Add<OrderedFloat<F>, Output = OrderedFloat<F>>,
    F: FloatCore,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
impl<F> std::ops::Sub for FloatCost<F>
where
    OrderedFloat<F>: std::ops::Sub<OrderedFloat<F>, Output = OrderedFloat<F>>,
    F: FloatCore,
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}
impl<F> std::ops::Mul for FloatCost<F>
where
    OrderedFloat<F>: std::ops::Mul<OrderedFloat<F>, Output = OrderedFloat<F>>,
    F: FloatCore,
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl<F> std::ops::AddAssign for FloatCost<F>
where
    OrderedFloat<F>: std::ops::AddAssign,
    F: FloatCore,
{
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
impl<F> SaturatingAdd for FloatCost<F>
where
    OrderedFloat<F>: std::ops::Add<OrderedFloat<F>, Output = OrderedFloat<F>>,
    F: FloatCore,
{
    fn saturating_add(&self, rhs: &Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl<F> Zero for FloatCost<F>
where
    F: FloatCore,
{
    #[inline(always)]
    fn is_zero(&self) -> bool {
        self.0 == OrderedFloat::zero()
    }
    #[inline(always)]
    fn zero() -> Self {
        Self(OrderedFloat::zero())
    }
}
impl<F> One for FloatCost<F>
where
    F: FloatCore,
{
    #[inline(always)]
    fn one() -> Self {
        Self(OrderedFloat::one())
    }
}
impl<F> UpperBounded for FloatCost<F>
where
    F: FloatCore,
{
    fn max_value() -> Self {
        Self(OrderedFloat::<F>::infinity())
    }
}

impl<F> PartialOrd for FloatCost<F>
where
    F: FloatCore,
{
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // `PartialOrd` is forwarded to `OrderedFloat`
        Some(self.0.cmp(&other.0))
    }
}
impl<F> Ord for FloatCost<F>
where
    OrderedFloat<F>: Ord,
    F: FloatCore,
{
    #[inline(always)]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // `Ord` is forwarded to `OrderedFloat`
        self.0.cmp(&other.0)
    }
}
impl<F> PartialEq for FloatCost<F>
where
    F: FloatCore,
{
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        let s: OrderedFloat<F> = self.0;
        let o: OrderedFloat<F> = other.0;
        s.eq(&o)
    }
}
impl<F> Eq for FloatCost<F> where F: FloatCore {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero() {
        assert!(FloatCost::new(0.0f32).is_zero());
        assert!(FloatCost::from_ordered_float(OrderedFloat(0.0f32)).is_zero());
    }

    #[test]
    fn order() {
        assert!(FloatCost::new(0.0f32) <= FloatCost::new(0.0f32));
        assert!(FloatCost::new(0.0f32) == FloatCost::new(0.0f32));
    }

    #[test]
    fn sum() {
        let mut f = FloatCost::new(0.0f32);
        f += FloatCost::new(1.0f32);
        f += FloatCost::new(1.0f32);
        assert!(f == FloatCost::new(2.0f32));
        f += FloatCost::infinity();
        assert!(f == FloatCost::max_value());
    }
}
