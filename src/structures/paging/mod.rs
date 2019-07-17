//! Abstractions for page tables and other paging related structures.
//!
//! Page tables translate virtual memory “pages” to physical memory “frames”.

pub use self::frame::PhysFrame;
pub use self::frame_alloc::{FrameAllocator, FrameDeallocator};
#[doc(no_inline)]
pub use self::mapper::MappedPageTable;
pub use self::mapper::{Mapper, MapperAllSizes};
#[cfg(target_arch = "x86_64")]
#[doc(no_inline)]
pub use self::mapper::{OffsetPageTable, RecursivePageTable};
pub use self::page::{Page, PageSize, Size1GiB, Size2MiB, Size4KiB};
pub use self::page_table::{PageTable, PageTableFlags};

pub mod frame;
mod frame_alloc;
pub mod mapper;
pub mod page;
pub mod page_table;
