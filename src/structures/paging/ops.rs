/// A checked equivalent to the `+` operator. Returning an `Option` signifying
/// if the operation result is within bounds.
///
/// Similarly to `std::ops::Add` note that `Rhs` is `Self` by default, but
/// this is not mandatory.
///
/// # Examples
///
/// ## Adding integers
///
/// ```
/// use x86_64::structures::paging::ops::CheckedAdd;
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
    type Output;

    fn checked_add(self, rhs: Rhs) -> Option<Self::Output>;
}

/// A checked equivalent to the `-` operator. Returning an `Option` signifying
/// if the operation result is within bounds.
///
/// Similarly to `std::ops::Sub` note that `Rhs` is `Self` by default, but
/// this is not mandatory.
///
/// # Examples
///
/// ## Subtracting integers
///
/// ```
/// use x86_64::structures::paging::ops::CheckedSub;
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
    type Output;

    fn checked_sub(self, rhs: Rhs) -> Option<Self::Output>;
}
