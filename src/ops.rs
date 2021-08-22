//! Additional arithmetic operators for working with addresses.

/// A checked equivalent to the `+` operator that returns `None` if the
/// operation would result in a panic or a wrapping overflow. Similar to
/// [`std::ops::Add`] an `Output` type _must_ be specified and a `Rhs` type
/// _may_ be specified (defaults to `Self`).
///
/// We define our own trait rather than use
/// [`num::CheckedAdd`](https://docs.rs/num/latest/num/trait.CheckedAdd.html)
/// due to num's implementation requiring that `Rhs` and `Output` both be `Self`.
///
/// # Example
///
/// ```
/// use x86_64::ops::CheckedAdd;
///
/// #[derive(Debug, PartialEq)]
/// struct Addr(u64);
///
/// impl CheckedAdd for Addr {
///     type Output = Self;
///
///     fn checked_add(self, other: Self) -> Option<Self> {
///         self.0.checked_add(other.0).map(Addr)
///     }
/// }
///
/// assert_eq!(Addr(1).checked_add(Addr(2)),
///            Some(Addr(3)));
///
/// // Overflowing add
/// assert_eq!(Addr(u64::MAX).checked_add(Addr(1)),
///            None);
/// ```
pub trait CheckedAdd<Rhs = Self> {
    /// The resulting type returned by the `checked_add` operation.
    type Output;

    /// Adds two numbers, checking for overflow. If overflow happens, None is returned.
    fn checked_add(self, rhs: Rhs) -> Option<Self::Output>;
}

/// A checked equivalent to the `-` operator that returns `None` if the
/// operation would result in a panic or a wrapping underflow. Similar to
/// [`std::ops::Sub`] an `Output` type _must_ be specified and a `Rhs` type
/// _may_ be specified (defaults to `Self`).
///
/// We define our own trait rather than use
/// [`num::CheckedSub`](https://docs.rs/num/latest/num/trait.CheckedSub.html)
/// due to num's implementation requiring that `Rhs` and `Output` both be `Self`.
///
/// # Example
///
/// ```
/// use x86_64::ops::CheckedSub;
///
/// #[derive(Debug, PartialEq)]
/// struct Addr(u64);
///
/// impl CheckedSub for Addr {
///     type Output = Self;
///
///     fn checked_sub(self, other: Self) -> Option<Self> {
///         self.0.checked_sub(other.0).map(Addr)
///     }
/// }
///
/// assert_eq!(Addr(3).checked_sub(Addr(2)),
///            Some(Addr(1)));
///
/// // Undeflowing sub
/// assert_eq!(Addr(u64::MIN).checked_sub(Addr(1)),
///            None);
/// ```
pub trait CheckedSub<Rhs = Self> {
    /// The resulting type returned by the `checked_sub` operation.
    type Output;

    /// Subtracts two numbers, checking for underflow. If underflow happens, None is returned.
    fn checked_sub(self, rhs: Rhs) -> Option<Self::Output>;
}
