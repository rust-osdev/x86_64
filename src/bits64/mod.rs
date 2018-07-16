//! Data structures and functions used by IA-32e but not Protected Mode.

pub mod paging;
pub mod rflags;
pub mod segmentation;
#[cfg(target_arch = "x86_64")]
pub mod registers;
#[cfg(target_arch = "x86_64")]
pub mod sgx;
pub mod syscall;
pub mod task;
