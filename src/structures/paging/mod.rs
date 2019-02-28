//! Abstractions for page tables and other paging related structures.
//!
//! Page tables translate virtual memory “pages” to physical memory “frames”.

pub use self::frame_alloc::{FrameAllocator, FrameDeallocator};
pub use self::page_table::{PageTable, PageTableFlags};
#[cfg(target_arch = "x86_64")]
pub use self::page::{Page, PageSize, Size4KiB, Size2MiB, Size1GiB};
pub use self::frame::PhysFrame;
pub use self::mapper::{Mapper, RecursivePageTable, MappedPageTable};

use crate::{PhysAddr, VirtAddr};
use core::fmt;
use core::marker::PhantomData;
use core::ops::{Add, AddAssign, Sub, SubAssign};
use ux::*;

pub mod page;
pub mod frame;
pub mod page_table;
pub mod mapper;
mod frame_alloc;
