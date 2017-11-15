pub mod control_regs;
pub mod descriptor;
pub mod dtables;
pub mod io;
pub mod irq;
pub mod msr;
pub mod paging;
pub mod flags;
pub mod segmentation;
pub mod task;
pub mod tlb;
pub mod time;

pub mod cpuid {
    pub use raw_cpuid::*;
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum PrivilegeLevel {
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
    Ring3 = 3,
}

#[cfg(feature = "performance-counter")]
pub mod perfcnt;

#[inline(always)]
pub unsafe fn halt() {
    asm!("hlt" :::: "volatile");
}
