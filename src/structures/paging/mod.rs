//! Abstractions for page tables and other paging related structures.
//!
//! Page tables translate virtual memory “pages” to physical memory “frames”.

pub use self::frame_alloc::*;
pub use self::page_table::*;
#[cfg(target_arch = "x86_64")]
pub use self::recursive::*;
pub use self::page::{Page, PageSize, Size4KiB, Size2MiB, Size1GiB};
pub use self::frame::PhysFrame;

use crate::{PhysAddr, VirtAddr};
use core::fmt;
use core::marker::PhantomData;
use core::ops::{Add, AddAssign, Sub, SubAssign};
use ux::*;

pub mod page;
pub mod frame;
mod frame_alloc;
mod page_table;
mod recursive;
