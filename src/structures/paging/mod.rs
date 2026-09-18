//! Abstractions for page tables and other paging related structures.
//!
//! Page tables translate virtual memory “pages” to physical memory “frames”.
//!
//! ## Virtual address width
//!
//! Enabling `default_virt_addr_57` changes the [`crate::VirtAddr`] type to use 57-bit
//! canonical addresses. The paging abstractions in this module currently retain their
//! 48-bit address-space assumptions, including the canonical-address gap handling in
//! page ranges and mapper range operations. Those operations do not yet provide full
//! five-level paging address-space coverage.

pub use self::frame::PhysFrame;
pub use self::frame_alloc::{FrameAllocator, FrameDeallocator};
#[doc(no_inline)]
pub use self::mapper::MappedPageTable;
#[cfg(all(feature = "instructions", target_arch = "x86_64"))]
#[doc(no_inline)]
pub use self::mapper::RecursivePageTable;
pub use self::mapper::{Mapper, Translate};
#[cfg(target_pointer_width = "64")]
#[doc(no_inline)]
pub use self::mapper::{OffsetPageTable, PhysOffset};
pub use self::page::{Page, PageSize, Size1GiB, Size2MiB, Size4KiB};
pub use self::page_table::{PageOffset, PageTable, PageTableFlags, PageTableIndex};

pub mod frame;
mod frame_alloc;
pub mod mapper;
pub mod page;
pub mod page_table;
