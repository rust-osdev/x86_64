//! Runtime virtual-address validity policy.

use core::convert::TryFrom;

#[cfg(all(feature = "instructions", target_arch = "x86_64"))]
mod instr;

use crate::structures::paging::PageTableIndex;

#[cfg(feature = "virt_addr_57")]
use super::VirtAddr57;
use super::{VirtAddr48, VirtAddrGeneric, VirtAddrNotValid, VirtAddrValidity};

/// The runtime virtual-address validity policy.
///
/// This policy checks the currently active address-space mode using a global cache of
/// `CR4.LA57`. The first operation that needs the active mode initializes the cache. The policy
/// type itself is available with the `virt_addr_rt` feature on all targets. Operations that do not
/// consult the active mode are available wherever the policy is available. Checked construction,
/// canonicalization, and address-producing arithmetic additionally require the `instructions`
/// feature and an `x86_64` target, and they must execute in Ring 0.
#[cfg_attr(feature = "doc_cfg", doc(cfg(feature = "virt_addr_rt")))]
#[cfg_attr(
    not(all(feature = "instructions", target_arch = "x86_64")),
    doc = r#"
Address-producing arithmetic is unavailable when the current address-space mode cannot be read:

```compile_fail
use x86_64::{RuntimeValidity, VirtAddrGeneric};

let address = VirtAddrGeneric::<RuntimeValidity>::zero();
let _ = address + 1u64;
```
"#
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuntimeValidity;

/// A virtual address checked against the current address-space mode when created.
///
/// This alias is available with the `virt_addr_rt` feature.
#[cfg_attr(feature = "doc_cfg", doc(cfg(feature = "virt_addr_rt")))]
pub type VirtAddrRT = VirtAddrGeneric<RuntimeValidity>;

impl crate::sealed::Sealed for RuntimeValidity {}

impl VirtAddrValidity for RuntimeValidity {
    fn bits() -> usize {
        #[cfg(all(feature = "instructions", target_arch = "x86_64"))]
        {
            instr::cached_virtual_address_bits()
        }

        #[cfg(not(all(feature = "instructions", target_arch = "x86_64")))]
        {
            // All callers of this function are expected to be disabled on non-x86_64 targets or
            // when the instructions feature is disabled.
            unreachable!(
                "runtime virtual-address width requires x86_64 and the instructions feature"
            )
        }
    }
}

impl VirtAddrGeneric<RuntimeValidity> {
    /// Returns the 9-bit level 5 page table index.
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub fn p5_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 12 >> 9 >> 9 >> 9 >> 9) as u16)
    }
}

impl From<VirtAddr48> for VirtAddrRT {
    #[inline]
    fn from(address: VirtAddr48) -> Self {
        unsafe { Self::new_unsafe(address.as_u64()) }
    }
}

#[cfg(feature = "virt_addr_57")]
impl From<VirtAddrRT> for VirtAddr57 {
    #[inline]
    fn from(address: VirtAddrRT) -> Self {
        unsafe { Self::new_unsafe(address.as_u64()) }
    }
}

impl TryFrom<VirtAddrRT> for VirtAddr48 {
    type Error = VirtAddrNotValid;

    #[inline]
    fn try_from(address: VirtAddrRT) -> Result<Self, Self::Error> {
        Self::try_new(address.as_u64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_fixed_conversions_preserve_or_check_values() {
        let address48 = VirtAddr48::new(0xffff_8000_0000_1234);
        let address_rt = VirtAddrRT::from(address48);

        assert_eq!(address_rt.as_u64(), address48.as_u64());
        assert_eq!(VirtAddr48::try_from(address_rt).unwrap(), address48);

        #[cfg(feature = "virt_addr_57")]
        {
            let address57 = VirtAddr57::from(address_rt);
            assert_eq!(address57.as_u64(), address48.as_u64());
        }
    }
}
