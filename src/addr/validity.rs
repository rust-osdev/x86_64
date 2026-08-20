//! Virtual-address validity and fixed-width validity policies.

use core::hash::Hash;

/// A policy for virtual-address validity.
///
/// This trait is sealed and cannot be implemented outside this crate. Three validities are
/// supported:
///
/// - [`FixedValidity<48>`]: 48-bit fixed width.
/// - [`FixedValidity<57>`]: 57-bit fixed width (requires the `virt_addr_57` feature).
/// - `RuntimeValidity`: Runtime validity (requires the `virt_addr_rt` feature).
///
/// This trait is used by [`VirtAddrGeneric`](super::VirtAddrGeneric) to construct different
/// virtual-address types.
///
/// # Examples
///
/// ```
/// use x86_64::addr::{FixedValidity, VirtAddrGeneric};
///
/// let addr = VirtAddrGeneric::<FixedValidity<48>>::new(0x1000);
/// ```
///
/// The set of validity policies is closed:
///
/// ```compile_fail
/// struct CustomValidity;
///
/// let _ = x86_64::addr::VirtAddrGeneric::<CustomValidity>::zero();
/// ```
#[cfg_attr(
    not(feature = "virt_addr_57"),
    doc = r#"
`FixedValidity<57>` requires the `virt_addr_57` feature:

```compile_fail
use x86_64::{FixedValidity, VirtAddrGeneric};

let _ = VirtAddrGeneric::<FixedValidity<57>>::zero();
```
"#
)]
pub trait VirtAddrValidity: crate::sealed::Sealed + Copy + Ord + Hash {
    /// Returns the number of valid bits in the virtual address.
    ///
    /// This function is not used in const contexts for `FixedValidity`, where the number of bits is
    /// known as a const parameter. It is used for common code that is generic over validity
    /// policies, including `RuntimeValidity`.
    fn bits() -> usize;
}

/// A fixed-width virtual-address validity policy.
///
/// `FixedValidity<48>` is always supported. `FixedValidity<57>` is supported with the
/// `virt_addr_57` feature.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedValidity<const BITS: usize>;

impl crate::sealed::Sealed for FixedValidity<48> {}

#[cfg(feature = "virt_addr_57")]
impl crate::sealed::Sealed for FixedValidity<57> {}

impl VirtAddrValidity for FixedValidity<48> {
    fn bits() -> usize {
        48
    }
}

#[cfg(feature = "virt_addr_57")]
impl VirtAddrValidity for FixedValidity<57> {
    fn bits() -> usize {
        57
    }
}

/// A [`VirtAddrValidity`] for which arithmetic operations are supported.
///
/// Enabled fixed validity policies always support arithmetic. `RuntimeValidity` supports
/// arithmetic when the `instructions` feature is enabled and the target is `x86_64`.
pub(crate) trait ArithmeticValidity: VirtAddrValidity {}

impl<const BITS: usize> ArithmeticValidity for FixedValidity<BITS> where
    FixedValidity<BITS>: VirtAddrValidity
{
}
