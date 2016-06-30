//! Data structures and functions used by IA-32e but not Protected Mode.

macro_rules! bit {
    ( $x:expr ) => {
        1 << $x
    };
}

macro_rules! check_flag {
    ($doc:meta, $fun:ident, $flag:ident) => (
        #[$doc]
        pub fn $fun(&self) -> bool {
            self.contains($flag)
        }
    )
}

macro_rules! is_bit_set {
    ($field:expr, $bit:expr) => (
        $field & (1 << $bit) > 0
    )
}

macro_rules! check_bit_fn {
    ($doc:meta, $fun:ident, $field:ident, $bit:expr) => (
        #[$doc]
        pub fn $fun(&self) -> bool {
            is_bit_set!(self.$field, $bit)
        }
    )
}

pub mod msr;
pub mod time;
pub mod irq;
pub mod paging;
pub mod task;
pub mod syscall;
pub mod sgx;
#[cfg(feature = "performance-counter")]
pub mod perfcnt;
pub mod cpuid {
    pub use raw_cpuid::*;
}
pub mod tlb;
