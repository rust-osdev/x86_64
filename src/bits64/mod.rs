//! Data structures and functions used by IA-32e but not Protected Mode.

macro_rules! check_flag {
    ($doc:meta, $fun:ident, $flag:ident) => (
        #[$doc]
        pub fn $fun(&self) -> bool {
            self.contains($flag)
        }
    )
}

pub mod irq;
pub mod rflags;
pub mod paging;
pub mod segmentation;
pub mod task;
pub mod syscall;
#[cfg(target_arch="x86-64")]
pub mod sgx;
