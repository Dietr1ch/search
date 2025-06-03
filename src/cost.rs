pub trait Cost:
    Copy
    + std::fmt::Debug
    + std::fmt::Display
    + PartialEq
    + core::cmp::Eq
    + PartialOrd
    + Ord
    + num_traits::SaturatingAdd
    + num_traits::bounds::UpperBounded
    + num_traits::Zero
    + num_traits::One
    + std::ops::Add<Self, Output = Self>
    + std::ops::Sub<Self, Output = Self>
    + std::ops::AddAssign
{
    #[inline(always)]
    fn valid(&self) -> bool {
        *self != num_traits::bounds::UpperBounded::max_value()
    }
}
