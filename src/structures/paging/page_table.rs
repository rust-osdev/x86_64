//! Abstractions for page tables and page table entries.

use core::fmt;
#[cfg(feature = "step_trait")]
use core::iter::Step;
use core::ops::{Index, IndexMut};
#[cfg(target_has_atomic = "64")]
use core::sync::atomic::AtomicU64;
#[cfg(any(feature = "memory_encryption", target_has_atomic = "64"))]
use core::sync::atomic::Ordering;

use super::{PageSize, PhysFrame, Size4KiB};
use crate::addr::PhysAddr;
#[cfg(feature = "memory_encryption")]
use crate::structures::mem_encrypt::PHYSICAL_ADDRESS_MASK;

use bitflags::bitflags;
use const_fn::const_fn;

/// The error returned by the `PageTableEntry::frame` method.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameError {
    /// The entry does not have the `PRESENT` flag set, so it isn't currently mapped to a frame.
    FrameNotPresent,
    /// The entry does have the `HUGE_PAGE` flag set. The `frame` method has a standard 4KiB frame
    /// as return type, so a huge frame can't be returned.
    HugeFrame,
}

/// A 64-bit page table entry.
#[derive(Clone)]
#[repr(transparent)]
pub struct PageTableEntry {
    entry: u64,
}

impl PageTableEntry {
    /// Creates an unused page table entry.
    #[inline]
    pub const fn new() -> Self {
        PageTableEntry { entry: 0 }
    }

    /// Returns the raw 64-bit value of this entry.
    ///
    /// Every 64-bit value is a valid `PageTableEntry` representation. Reserved
    /// bits are rejected by the hardware when a present entry is used, not when
    /// the entry is built.
    #[inline]
    pub const fn bits(&self) -> u64 {
        self.entry
    }

    /// Creates an entry from its raw 64-bit value.
    ///
    /// Every 64-bit value is a valid `PageTableEntry` representation. Reserved
    /// bits are rejected by the hardware when a present entry is used, not when
    /// the entry is built.
    #[inline]
    pub const fn from_bits(bits: u64) -> Self {
        PageTableEntry { entry: bits }
    }

    /// Returns whether this entry is zero.
    #[inline]
    pub const fn is_unused(&self) -> bool {
        self.entry == 0
    }

    /// Sets this entry to zero.
    #[inline]
    pub fn set_unused(&mut self) {
        self.entry = 0;
    }

    /// Returns the flags of this entry.
    #[inline]
    #[const_fn(cfg(not(feature = "memory_encryption")))]
    pub const fn flags(&self) -> PageTableFlags {
        PageTableFlags::from_bits_retain(self.entry & !Self::physical_address_mask())
    }

    /// Returns the physical address mapped by this entry, might be zero.
    #[inline]
    pub fn addr(&self) -> PhysAddr {
        PhysAddr::new(self.entry & Self::physical_address_mask())
    }

    /// Returns the physical frame mapped by this entry.
    ///
    /// Set `is_level_1_entry` to `true` if the `PageTableEntry` is from a
    /// level 1 page table.
    ///
    /// Returns the following errors:
    ///
    /// - `FrameError::FrameNotPresent` if the entry doesn't have the `PRESENT` flag set.
    /// - `FrameError::HugeFrame` if the entry has the `HUGE_PAGE` flag set (for huge pages the
    ///   `addr` function must be used) and `is_level_1_entry` is `false`
    #[inline]
    pub fn frame(&self, is_level_1_entry: bool) -> Result<PhysFrame, FrameError> {
        if !self.flags().contains(PageTableFlags::PRESENT) {
            Err(FrameError::FrameNotPresent)
        } else if !is_level_1_entry && self.flags().contains(PageTableFlags::HUGE_PAGE) {
            Err(FrameError::HugeFrame)
        } else {
            Ok(PhysFrame::containing_address(self.addr()))
        }
    }

    /// Map the entry to the specified physical address with the specified flags.
    #[inline]
    pub fn set_addr(&mut self, addr: PhysAddr, flags: PageTableFlags) {
        assert!(addr.is_aligned(Size4KiB::SIZE));
        self.entry = (addr.as_u64()) | flags.bits();
    }

    /// Map the entry to the specified physical frame with the specified flags.
    #[inline]
    pub fn set_frame(&mut self, frame: PhysFrame, flags: PageTableFlags) {
        self.set_addr(frame.start_address(), flags)
    }

    /// Sets the flags of this entry.
    #[inline]
    pub fn set_flags(&mut self, flags: PageTableFlags) {
        self.entry = self.addr().as_u64() | flags.bits();
    }

    #[inline(always)]
    #[cfg(not(feature = "memory_encryption"))]
    const fn physical_address_mask() -> u64 {
        0x000f_ffff_ffff_f000
    }

    #[inline(always)]
    #[cfg(feature = "memory_encryption")]
    fn physical_address_mask() -> u64 {
        PHYSICAL_ADDRESS_MASK.load(Ordering::Relaxed) & !0xfff
    }
}

impl Default for PageTableEntry {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("PageTableEntry");
        f.field("addr", &self.addr());
        f.field("flags", &self.flags());
        f.finish()
    }
}

bitflags! {
    /// Possible flags for a page table entry.
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct PageTableFlags: u64 {
        /// Specifies whether the mapped frame or page table is loaded in memory.
        const PRESENT =         1;
        /// Controls whether writes to the mapped frames are allowed.
        ///
        /// If this bit is unset in a level 1 page table entry, the mapped frame is read-only.
        /// If this bit is unset in a higher level page table entry the complete range of mapped
        /// pages is read-only.
        const WRITABLE =        1 << 1;
        /// Controls whether accesses from userspace (i.e. ring 3) are permitted.
        const USER_ACCESSIBLE = 1 << 2;
        /// If this bit is set, a “write-through” policy is used for the cache, else a “write-back”
        /// policy is used. This referred to as the page-level write-through (PWT) bit.
        const WRITE_THROUGH =   1 << 3;
        /// Disables caching for the pointed entry if it is cacheable. This referred to as the
        /// page-level cache disable (PCD) bit.
        const NO_CACHE =        1 << 4;
        /// Set by the CPU when the mapped frame or page table is accessed.
        const ACCESSED =        1 << 5;
        /// Set by the CPU on a write to the mapped frame.
        const DIRTY =           1 << 6;
        /// Specifies that the entry maps a huge frame instead of a page table. This is the same bit
        /// as `PAT_4KIB_PAGE`.
        const HUGE_PAGE =       1 << 7;
        /// This is the PAT bit for page table entries that point to 4KiB pages. This is the same
        /// bit as `HUGE_PAGE`.
        const PAT_4KIB_PAGE =   1 << 7;
        /// Indicates that the mapping is present in all address spaces, so it isn't flushed from
        /// the TLB on an address space switch.
        const GLOBAL =          1 << 8;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_9 =           1 << 9;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_10 =          1 << 10;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_11 =          1 << 11;
        /// This is the PAT bit for page table entries that point to huge pages.
        const PAT_HUGE_PAGE =   1 << 12;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_52 =          1 << 52;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_53 =          1 << 53;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_54 =          1 << 54;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_55 =          1 << 55;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_56 =          1 << 56;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_57 =          1 << 57;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_58 =          1 << 58;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_59 =          1 << 59;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_60 =          1 << 60;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_61 =          1 << 61;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_62 =          1 << 62;
        /// Forbid code execution from the mapped frames.
        ///
        /// Can be only used when the no-execute page protection feature is enabled in the EFER
        /// register.
        const NO_EXECUTE =      1 << 63;
    }
}

/// The raw bit operations shared by [`PageTableEntry`] and its atomic
/// counterpart.
///
/// Only [`bits`](Self::bits) has to be implemented. The other methods are
/// derived from the raw value. On [`PageTableEntry`] the inherent `const`
/// methods take precedence when they are called directly.
pub trait PageTableEntryBits: Sized {
    /// Returns the raw 64-bit value of this entry.
    fn bits(&self) -> u64;

    /// Returns whether this entry is zero.
    #[inline]
    fn is_unused(&self) -> bool {
        self.bits() == 0
    }

    /// Returns the flags of this entry.
    #[inline]
    fn flags(&self) -> PageTableFlags {
        PageTableEntry::from_bits(self.bits()).flags()
    }

    /// Returns the physical address mapped by this entry, might be zero.
    #[inline]
    fn addr(&self) -> PhysAddr {
        PageTableEntry::from_bits(self.bits()).addr()
    }

    /// Returns the physical frame mapped by this entry.
    ///
    /// Set `is_level_1_entry` to `true` if the entry is from a level 1 page
    /// table.
    ///
    /// Returns [`FrameError::FrameNotPresent`] if the entry doesn't have the
    /// `PRESENT` flag set, and [`FrameError::HugeFrame`] if the entry has the
    /// `HUGE_PAGE` flag set (for huge pages [`addr`](Self::addr) must be used)
    /// and `is_level_1_entry` is `false`.
    #[inline]
    fn frame(&self, is_level_1_entry: bool) -> Result<PhysFrame, FrameError> {
        PageTableEntry::from_bits(self.bits()).frame(is_level_1_entry)
    }
}

/// The mutable raw bit operations shared by [`PageTableEntry`] and its atomic
/// counterpart.
///
/// Only [`set_bits`](Self::set_bits) has to be implemented. The other methods
/// are derived from it. On [`PageTableEntry`] the inherent methods take
/// precedence when they are called directly.
///
/// The setters are not a single atomic read-modify-write. On an atomic entry
/// they use relaxed loads and stores, so they are intended for entries that
/// are not being modified concurrently, for example while a table is being
/// built. Use the atomic operations directly for live entries.
pub trait PageTableEntryBitsMut: PageTableEntryBits {
    /// Sets the raw 64-bit value of this entry.
    fn set_bits(&mut self, bits: u64);

    /// Sets this entry to zero.
    #[inline]
    fn set_unused(&mut self) {
        self.set_bits(0)
    }

    /// Sets the flags of this entry.
    #[inline]
    fn set_flags(&mut self, flags: PageTableFlags) {
        let mut entry = PageTableEntry::from_bits(self.bits());
        entry.set_flags(flags);
        self.set_bits(entry.bits());
    }

    /// Maps the entry to the specified physical address with the specified
    /// flags.
    #[inline]
    fn set_addr(&mut self, addr: PhysAddr, flags: PageTableFlags) {
        let mut entry = PageTableEntry::from_bits(self.bits());
        entry.set_addr(addr, flags);
        self.set_bits(entry.bits());
    }

    /// Maps the entry to the specified physical frame with the specified flags.
    #[inline]
    fn set_frame(&mut self, frame: PhysFrame, flags: PageTableFlags) {
        let mut entry = PageTableEntry::from_bits(self.bits());
        entry.set_frame(frame, flags);
        self.set_bits(entry.bits());
    }
}

impl PageTableEntryBits for PageTableEntry {
    #[inline]
    fn bits(&self) -> u64 {
        self.entry
    }
}

impl PageTableEntryBitsMut for PageTableEntry {
    #[inline]
    fn set_bits(&mut self, bits: u64) {
        self.entry = bits;
    }
}

/// An atomic 64-bit page table entry.
///
/// This is the atomic counterpart of [`PageTableEntry`]. It is a
/// `#[repr(transparent)]` wrapper around `AtomicU64`, so it has the same layout
/// as `AtomicU64`. On x86_64 it also has the same size and alignment as
/// [`PageTableEntry`].
///
/// The `PageTableEntry`-shaped methods (`is_unused`, `flags`, `addr`, `frame`
/// and the setters) use relaxed loads and stores. They forward to the
/// [`PageTableEntryBits`] and [`PageTableEntryBitsMut`] implementations, so the
/// same methods are also available in generic code. Use
/// [`load`](Self::load), [`store`](Self::store), [`swap`](Self::swap) or the
/// compare-exchange methods when an ordering matters.
#[cfg(target_has_atomic = "64")]
#[repr(transparent)]
pub struct AtomicPageTableEntry(AtomicU64);

#[cfg(target_has_atomic = "64")]
impl AtomicPageTableEntry {
    /// Creates a zeroed entry.
    #[inline]
    pub const fn new() -> Self {
        Self(AtomicU64::new(0))
    }

    /// Creates an entry with the given raw 64-bit value.
    #[inline]
    pub const fn from_bits(bits: u64) -> Self {
        Self(AtomicU64::new(bits))
    }

    /// Returns the raw 64-bit value of this entry.
    ///
    /// This uses [`Ordering::Relaxed`]. Use [`load`](Self::load) to choose an
    /// ordering.
    #[inline]
    pub fn bits(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }

    /// Loads the entry with the given ordering.
    #[inline]
    pub fn load(&self, order: Ordering) -> PageTableEntry {
        PageTableEntry::from_bits(self.0.load(order))
    }

    /// Stores an entry with the given ordering.
    #[inline]
    pub fn store(&self, entry: PageTableEntry, order: Ordering) {
        self.0.store(entry.bits(), order);
    }

    /// Stores an entry and returns the previous value.
    #[inline]
    pub fn swap(&self, entry: PageTableEntry, order: Ordering) -> PageTableEntry {
        PageTableEntry::from_bits(self.0.swap(entry.bits(), order))
    }

    /// Compares and exchanges the entry with the given orderings.
    ///
    /// # Panics
    ///
    /// Panics if `failure` is [`Ordering::Release`] or [`Ordering::AcqRel`].
    #[inline]
    pub fn compare_exchange(
        &self,
        current: PageTableEntry,
        new: PageTableEntry,
        success: Ordering,
        failure: Ordering,
    ) -> Result<PageTableEntry, PageTableEntry> {
        match self
            .0
            .compare_exchange(current.bits(), new.bits(), success, failure)
        {
            Ok(bits) => Ok(PageTableEntry::from_bits(bits)),
            Err(bits) => Err(PageTableEntry::from_bits(bits)),
        }
    }

    /// Compares and exchanges the entry, allowing spurious failures.
    ///
    /// # Panics
    ///
    /// Panics if `failure` is [`Ordering::Release`] or [`Ordering::AcqRel`].
    #[inline]
    pub fn compare_exchange_weak(
        &self,
        current: PageTableEntry,
        new: PageTableEntry,
        success: Ordering,
        failure: Ordering,
    ) -> Result<PageTableEntry, PageTableEntry> {
        match self
            .0
            .compare_exchange_weak(current.bits(), new.bits(), success, failure)
        {
            Ok(bits) => Ok(PageTableEntry::from_bits(bits)),
            Err(bits) => Err(PageTableEntry::from_bits(bits)),
        }
    }

    /// Bitwise ANDs the entry with `bits` and returns the previous value.
    #[inline]
    pub fn fetch_and(&self, bits: u64, order: Ordering) -> PageTableEntry {
        PageTableEntry::from_bits(self.0.fetch_and(bits, order))
    }

    /// Bitwise ORs the entry with `bits` and returns the previous value.
    #[inline]
    pub fn fetch_or(&self, bits: u64, order: Ordering) -> PageTableEntry {
        PageTableEntry::from_bits(self.0.fetch_or(bits, order))
    }

    /// Returns a mutable reference to the entry.
    #[inline]
    pub fn get_mut(&mut self) -> &mut PageTableEntry {
        // SAFETY: `PageTableEntry` is `#[repr(transparent)]` over `u64`, and
        // `AtomicU64` has the same size as `u64` with an alignment at least as
        // strict.
        unsafe { &mut *(self.0.get_mut() as *mut u64).cast::<PageTableEntry>() }
    }

    /// Returns a pointer to the entry.
    #[inline]
    pub fn as_ptr(&self) -> *mut PageTableEntry {
        self.0.as_ptr().cast()
    }

    /// Creates a reference to an atomic entry from a pointer.
    ///
    /// # Safety
    ///
    /// The pointer must be valid, properly aligned, and point to a
    /// `PageTableEntry` that is only accessed atomically for the lifetime `'a`.
    #[inline]
    pub unsafe fn from_ptr<'a>(ptr: *mut PageTableEntry) -> &'a Self {
        // SAFETY: guaranteed by the caller; `Self` has the same layout as
        // `PageTableEntry`.
        unsafe { &*ptr.cast::<Self>() }
    }

    /// Consumes the entry and returns its value.
    #[inline]
    pub fn into_inner(self) -> PageTableEntry {
        PageTableEntry::from_bits(self.0.into_inner())
    }

    /// Returns whether this entry is zero.
    ///
    /// This forwards to [`PageTableEntryBits::is_unused`].
    #[inline]
    pub fn is_unused(&self) -> bool {
        PageTableEntryBits::is_unused(self)
    }

    /// Returns the flags of this entry.
    ///
    /// This forwards to [`PageTableEntryBits::flags`].
    #[inline]
    pub fn flags(&self) -> PageTableFlags {
        PageTableEntryBits::flags(self)
    }

    /// Returns the physical address mapped by this entry, might be zero.
    ///
    /// This forwards to [`PageTableEntryBits::addr`].
    #[inline]
    pub fn addr(&self) -> PhysAddr {
        PageTableEntryBits::addr(self)
    }

    /// Returns the physical frame mapped by this entry.
    ///
    /// This forwards to [`PageTableEntryBits::frame`].
    #[inline]
    pub fn frame(&self, is_level_1_entry: bool) -> Result<PhysFrame, FrameError> {
        PageTableEntryBits::frame(self, is_level_1_entry)
    }

    /// Sets this entry to zero.
    ///
    /// This forwards to [`PageTableEntryBitsMut::set_unused`].
    #[inline]
    pub fn set_unused(&mut self) {
        PageTableEntryBitsMut::set_unused(self)
    }

    /// Sets the flags of this entry.
    ///
    /// This forwards to [`PageTableEntryBitsMut::set_flags`].
    #[inline]
    pub fn set_flags(&mut self, flags: PageTableFlags) {
        PageTableEntryBitsMut::set_flags(self, flags)
    }

    /// Maps the entry to the specified physical address with the specified
    /// flags.
    ///
    /// This forwards to [`PageTableEntryBitsMut::set_addr`].
    #[inline]
    pub fn set_addr(&mut self, addr: PhysAddr, flags: PageTableFlags) {
        PageTableEntryBitsMut::set_addr(self, addr, flags)
    }

    /// Maps the entry to the specified physical frame with the specified flags.
    ///
    /// This forwards to [`PageTableEntryBitsMut::set_frame`].
    #[inline]
    pub fn set_frame(&mut self, frame: PhysFrame, flags: PageTableFlags) {
        PageTableEntryBitsMut::set_frame(self, frame, flags)
    }
}

#[cfg(target_has_atomic = "64")]
impl Clone for AtomicPageTableEntry {
    #[inline]
    fn clone(&self) -> Self {
        Self::from_bits(self.bits())
    }
}

#[cfg(target_has_atomic = "64")]
impl Default for AtomicPageTableEntry {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_has_atomic = "64")]
impl fmt::Debug for AtomicPageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("AtomicPageTableEntry")
            .field(&self.load(Ordering::Relaxed))
            .finish()
    }
}

#[cfg(target_has_atomic = "64")]
impl PageTableEntryBits for AtomicPageTableEntry {
    #[inline]
    fn bits(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }
}

#[cfg(target_has_atomic = "64")]
impl PageTableEntryBitsMut for AtomicPageTableEntry {
    #[inline]
    fn set_bits(&mut self, bits: u64) {
        self.0.store(bits, Ordering::Relaxed);
    }
}

/// The number of entries in a page table.
const ENTRY_COUNT: usize = 512;

/// Represents a page table.
///
/// Always page-sized.
///
/// This struct implements the `Index` and `IndexMut` traits, so the entries can be accessed
/// through index operations. For example, `page_table[15]` returns the 16th page table entry.
///
/// Note that while this type implements [`Clone`], the users must be careful not to introduce
/// mutable aliasing by using the cloned page tables.
#[repr(align(4096))]
#[repr(C)]
#[derive(Clone)]
pub struct PageTable {
    entries: [PageTableEntry; ENTRY_COUNT],
}

impl PageTable {
    /// Creates an empty page table.
    #[inline]
    pub const fn new() -> Self {
        const EMPTY: PageTableEntry = PageTableEntry::new();
        PageTable {
            entries: [EMPTY; ENTRY_COUNT],
        }
    }

    /// Clears all entries.
    #[inline]
    pub fn zero(&mut self) {
        for entry in self.iter_mut() {
            entry.set_unused();
        }
    }

    /// Returns an iterator over the entries of the page table.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &PageTableEntry> {
        (0..512).map(move |i| &self.entries[i])
    }

    /// Returns an iterator that allows modifying the entries of the page table.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut PageTableEntry> {
        // Note that we intentionally don't just return `self.entries.iter()`:
        // Some users may choose to create a reference to a page table at
        // `0xffff_ffff_ffff_f000`. This causes problems because calculating
        // the end pointer of the page tables causes an overflow. Therefore
        // creating page tables at that address is unsound and must be avoided.
        // Unfortunately creating such page tables is quite common when
        // recursive page tables are used, so we try to avoid calculating the
        // end pointer if possible. `core::slice::Iter` calculates the end
        // pointer to determine when it should stop yielding elements. Because
        // we want to avoid calculating the end pointer, we don't use
        // `core::slice::Iter`, we implement our own iterator that doesn't
        // calculate the end pointer. This doesn't make creating page tables at
        // that address sound, but it avoids some easy to trigger
        // miscompilations.
        let ptr = self.entries.as_mut_ptr();
        (0..512).map(move |i| unsafe { &mut *ptr.add(i) })
    }

    /// Checks if the page table is empty (all entries are zero).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.iter().all(|entry| entry.is_unused())
    }
}

impl Index<usize> for PageTable {
    type Output = PageTableEntry;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for PageTable {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl Index<PageTableIndex> for PageTable {
    type Output = PageTableEntry;

    #[inline]
    fn index(&self, index: PageTableIndex) -> &Self::Output {
        &self.entries[usize::from(index)]
    }
}

impl IndexMut<PageTableIndex> for PageTable {
    #[inline]
    fn index_mut(&mut self, index: PageTableIndex) -> &mut Self::Output {
        &mut self.entries[usize::from(index)]
    }
}

impl Default for PageTable {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for PageTable {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.entries[..].fmt(f)
    }
}

/// An atomic page table.
///
/// This is the atomic counterpart of [`PageTable`] and exposes the same API:
/// [`new`](Self::new), [`zero`](Self::zero), [`iter`](Self::iter),
/// [`iter_mut`](Self::iter_mut), [`is_empty`](Self::is_empty), indexing,
/// [`Default`], [`Clone`] and [`Debug`].
///
/// It is `#[repr(C, align(4096))]` over 512 [`AtomicPageTableEntry`] values, so
/// it has the same size and alignment as [`PageTable`].
///
/// The CPU can update the `ACCESSED` and `DIRTY` bits of live entries on its
/// own, so entries of a live table must only be accessed atomically.
///
/// Note that while this type implements [`Clone`], a cloned table is a separate
/// copy of the entries.
#[cfg(target_has_atomic = "64")]
#[repr(align(4096))]
#[repr(C)]
#[derive(Clone)]
pub struct AtomicPageTable {
    entries: [AtomicPageTableEntry; ENTRY_COUNT],
}

#[cfg(target_has_atomic = "64")]
impl AtomicPageTable {
    /// Creates an empty page table.
    #[inline]
    pub const fn new() -> Self {
        Self {
            entries: [const { AtomicPageTableEntry::new() }; ENTRY_COUNT],
        }
    }

    /// Clears all entries.
    #[inline]
    pub fn zero(&mut self) {
        for entry in self.iter_mut() {
            entry.set_unused();
        }
    }

    /// Returns an iterator over the entries of the page table.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &AtomicPageTableEntry> {
        (0..ENTRY_COUNT).map(move |i| &self.entries[i])
    }

    /// Returns an iterator that allows modifying the entries of the page table.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut AtomicPageTableEntry> {
        // See `PageTable::iter_mut` for why this does not use `slice::Iter`.
        let ptr = self.entries.as_mut_ptr();
        (0..ENTRY_COUNT).map(move |i| {
            // SAFETY: `i` is in `0..ENTRY_COUNT`, and the iterator borrows the
            // table mutably for its lifetime.
            unsafe { &mut *ptr.add(i) }
        })
    }

    /// Checks if the page table is empty (all entries are zero).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.iter().all(|entry| entry.is_unused())
    }
}

#[cfg(target_has_atomic = "64")]
impl Index<usize> for AtomicPageTable {
    type Output = AtomicPageTableEntry;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

#[cfg(target_has_atomic = "64")]
impl IndexMut<usize> for AtomicPageTable {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

#[cfg(target_has_atomic = "64")]
impl Index<PageTableIndex> for AtomicPageTable {
    type Output = AtomicPageTableEntry;

    #[inline]
    fn index(&self, index: PageTableIndex) -> &Self::Output {
        &self.entries[usize::from(index)]
    }
}

#[cfg(target_has_atomic = "64")]
impl IndexMut<PageTableIndex> for AtomicPageTable {
    #[inline]
    fn index_mut(&mut self, index: PageTableIndex) -> &mut Self::Output {
        &mut self.entries[usize::from(index)]
    }
}

#[cfg(target_has_atomic = "64")]
impl Default for AtomicPageTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_has_atomic = "64")]
impl fmt::Debug for AtomicPageTable {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.entries[..].fmt(f)
    }
}

/// A 9-bit index into a page table.
///
/// Can be used to select one of the 512 entries of a page table.
///
/// Guaranteed to only ever contain 0..512.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PageTableIndex(u16);

impl PageTableIndex {
    /// Creates a new index from the given `u16`. Panics if the given value is >=512.
    #[inline]
    pub const fn new(index: u16) -> Self {
        assert!((index as usize) < ENTRY_COUNT);
        Self(index)
    }

    /// Creates a new index from the given `u16`. Throws away bits if the value is >=512.
    #[inline]
    pub const fn new_truncate(index: u16) -> Self {
        Self(index % ENTRY_COUNT as u16)
    }

    #[inline]
    pub(crate) const fn into_u64(self) -> u64 {
        self.0 as u64
    }
}

impl From<PageTableIndex> for u16 {
    #[inline]
    fn from(index: PageTableIndex) -> Self {
        index.0
    }
}

impl From<PageTableIndex> for u32 {
    #[inline]
    fn from(index: PageTableIndex) -> Self {
        u32::from(index.0)
    }
}

impl From<PageTableIndex> for u64 {
    #[inline]
    fn from(index: PageTableIndex) -> Self {
        index.into_u64()
    }
}

impl From<PageTableIndex> for usize {
    #[inline]
    fn from(index: PageTableIndex) -> Self {
        usize::from(index.0)
    }
}

#[cfg(feature = "step_trait")]
impl Step for PageTableIndex {
    #[inline]
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        Step::steps_between(&start.0, &end.0)
    }

    #[inline]
    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        let idx = usize::from(start).checked_add(count)?;
        (idx < ENTRY_COUNT).then(|| Self::new(idx as u16))
    }

    #[inline]
    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        let idx = usize::from(start).checked_sub(count)?;
        Some(Self::new(idx as u16))
    }

    #[inline]
    fn forward_overflowing(start: Self, count: usize) -> (Self, bool) {
        match Self::forward_checked(start, count) {
            Some(next) => (next, false),
            None => (start, true),
        }
    }

    #[inline]
    fn backward_overflowing(start: Self, count: usize) -> (Self, bool) {
        match Self::backward_checked(start, count) {
            Some(next) => (next, false),
            None => (start, true),
        }
    }
}

/// A 12-bit offset into a 4KiB Page.
///
/// This type is returned by the `VirtAddr::page_offset` method.
///
/// Guaranteed to only ever contain 0..4096.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PageOffset(u16);

impl PageOffset {
    /// Creates a new offset from the given `u16`. Panics if the passed value is >=4096.
    #[inline]
    pub fn new(offset: u16) -> Self {
        assert!(offset < (1 << 12));
        Self(offset)
    }

    /// Creates a new offset from the given `u16`. Throws away bits if the value is >=4096.
    #[inline]
    pub const fn new_truncate(offset: u16) -> Self {
        Self(offset % (1 << 12))
    }
}

impl From<PageOffset> for u16 {
    #[inline]
    fn from(offset: PageOffset) -> Self {
        offset.0
    }
}

impl From<PageOffset> for u32 {
    #[inline]
    fn from(offset: PageOffset) -> Self {
        u32::from(offset.0)
    }
}

impl From<PageOffset> for u64 {
    #[inline]
    fn from(offset: PageOffset) -> Self {
        u64::from(offset.0)
    }
}

impl From<PageOffset> for usize {
    #[inline]
    fn from(offset: PageOffset) -> Self {
        usize::from(offset.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A value between 1 and 4.
pub enum PageTableLevel {
    /// Represents the level for a page table.
    One = 1,
    /// Represents the level for a page directory.
    Two,
    /// Represents the level for a page-directory pointer.
    Three,
    /// Represents the level for a page-map level-4.
    Four,
}

impl PageTableLevel {
    /// Returns the next lower level or `None` for level 1
    pub const fn next_lower_level(self) -> Option<Self> {
        match self {
            PageTableLevel::Four => Some(PageTableLevel::Three),
            PageTableLevel::Three => Some(PageTableLevel::Two),
            PageTableLevel::Two => Some(PageTableLevel::One),
            PageTableLevel::One => None,
        }
    }

    /// Returns the next higher level or `None` for level 4
    pub const fn next_higher_level(self) -> Option<Self> {
        match self {
            PageTableLevel::Four => None,
            PageTableLevel::Three => Some(PageTableLevel::Four),
            PageTableLevel::Two => Some(PageTableLevel::Three),
            PageTableLevel::One => Some(PageTableLevel::Two),
        }
    }

    /// Returns the alignment for the address space described by a table of this level.
    pub const fn table_address_space_alignment(self) -> u64 {
        1u64 << (self as u8 * 9 + 12)
    }

    /// Returns the alignment for the address space described by an entry in a table of this level.
    pub const fn entry_address_space_alignment(self) -> u64 {
        1u64 << (((self as u8 - 1) * 9) + 12)
    }
}

#[cfg(test)]
mod entry_tests {
    use super::*;

    #[test]
    fn page_table_entry_bits_roundtrip() {
        let bits = 0x0000_0000_0000_1001;
        let entry = PageTableEntry::from_bits(bits);
        assert_eq!(entry.bits(), bits);
        assert_eq!(PageTableEntry::new().bits(), 0);
    }

    #[test]
    fn page_table_entry_shared_trait() {
        fn check<E: PageTableEntryBitsMut>(entry: &mut E) {
            assert!(entry.is_unused());

            entry.set_addr(
                PhysAddr::new(0x1000),
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            );
            assert!(!entry.is_unused());
            assert_eq!(entry.addr(), PhysAddr::new(0x1000));
            assert!(entry.flags().contains(PageTableFlags::PRESENT));
            assert!(entry.flags().contains(PageTableFlags::WRITABLE));

            entry.set_frame(
                PhysFrame::containing_address(PhysAddr::new(0x2000)),
                PageTableFlags::PRESENT,
            );
            assert_eq!(entry.addr(), PhysAddr::new(0x2000));
            assert_eq!(
                entry.frame(true).unwrap(),
                PhysFrame::containing_address(PhysAddr::new(0x2000))
            );

            entry.set_flags(PageTableFlags::PRESENT);
            assert!(entry.flags().contains(PageTableFlags::PRESENT));
            assert!(!entry.flags().contains(PageTableFlags::WRITABLE));

            entry.set_unused();
            assert!(entry.is_unused());
        }

        let mut entry = PageTableEntry::new();
        check(&mut entry);

        #[cfg(target_has_atomic = "64")]
        {
            let mut entry = AtomicPageTableEntry::new();
            check(&mut entry);
        }
    }

    #[cfg(target_has_atomic = "64")]
    #[test]
    fn atomic_page_table_entry_operations() {
        use core::sync::atomic::Ordering;

        let mut entry = AtomicPageTableEntry::new();
        assert_eq!(entry.load(Ordering::Relaxed).bits(), 0);

        entry.store(PageTableEntry::from_bits(0x1001), Ordering::Relaxed);
        assert_eq!(entry.load(Ordering::Relaxed).bits(), 0x1001);

        let old = entry.swap(PageTableEntry::from_bits(0x2001), Ordering::AcqRel);
        assert_eq!(old.bits(), 0x1001);

        let current = PageTableEntry::from_bits(0x2001);
        let new = PageTableEntry::from_bits(0x3001);
        let result = entry.compare_exchange(
            current.clone(),
            new.clone(),
            Ordering::AcqRel,
            Ordering::Acquire,
        );
        assert_eq!(result.unwrap().bits(), 0x2001);
        let result = entry.compare_exchange(current, new, Ordering::AcqRel, Ordering::Acquire);
        assert_eq!(result.unwrap_err().bits(), 0x3001);

        let old = entry.fetch_or(PageTableFlags::WRITABLE.bits(), Ordering::Relaxed);
        assert_eq!(old.bits(), 0x3001);
        assert_eq!(
            entry.load(Ordering::Relaxed).bits(),
            0x3001 | PageTableFlags::WRITABLE.bits()
        );

        let old = entry.fetch_and(!PageTableFlags::WRITABLE.bits(), Ordering::Relaxed);
        assert_eq!(old.bits(), 0x3001 | PageTableFlags::WRITABLE.bits());
        assert_eq!(entry.load(Ordering::Relaxed).bits(), 0x3001);

        entry.get_mut().set_unused();
        assert_eq!(entry.load(Ordering::Relaxed).bits(), 0);

        entry.set_addr(PhysAddr::new(0x4000), PageTableFlags::PRESENT);
        assert!(!entry.is_unused());
        assert_eq!(entry.addr(), PhysAddr::new(0x4000));
        assert!(entry.flags().contains(PageTableFlags::PRESENT));

        let entry = AtomicPageTableEntry::from_bits(7);
        assert_eq!(entry.into_inner().bits(), 7);
    }

    #[cfg(target_has_atomic = "64")]
    #[test]
    fn atomic_page_table_entry_pointers() {
        use core::sync::atomic::Ordering;

        let entry = AtomicPageTableEntry::new();
        let ptr = entry.as_ptr();
        assert_eq!(ptr as usize, &entry as *const _ as usize);

        // SAFETY: the pointer comes from `entry` and outlives this call.
        let borrowed = unsafe { AtomicPageTableEntry::from_ptr(ptr) };
        borrowed.store(PageTableEntry::from_bits(0x1001), Ordering::Relaxed);
        assert_eq!(entry.load(Ordering::Relaxed).bits(), 0x1001);
    }

    #[cfg(target_has_atomic = "64")]
    #[test]
    fn atomic_page_table_api() {
        use core::sync::atomic::Ordering;

        assert_eq!(core::mem::size_of::<AtomicPageTable>(), 4096);
        assert_eq!(core::mem::align_of::<AtomicPageTable>(), 4096);

        let mut table = AtomicPageTable::new();
        assert!(table.is_empty());

        table[0].store(PageTableEntry::from_bits(0x1001), Ordering::Relaxed);
        assert!(!table.is_empty());
        assert_eq!(
            table[0].load(Ordering::Relaxed).addr(),
            PhysAddr::new(0x1000)
        );

        table[PageTableIndex::new(1)].store(PageTableEntry::from_bits(0x2001), Ordering::Relaxed);
        assert_eq!(table.iter().filter(|entry| !entry.is_unused()).count(), 2);

        for entry in table.iter_mut() {
            entry.set_unused();
        }
        assert!(table.is_empty());

        table[0].store(PageTableEntry::from_bits(0x1001), Ordering::Relaxed);
        table.zero();
        assert!(table.is_empty());

        let cloned = table.clone();
        assert!(cloned.is_empty());
        assert_eq!(cloned[0].load(Ordering::Relaxed).bits(), 0);

        let _ = format!("{:?}", table);
    }

    #[cfg(all(target_arch = "x86_64", target_has_atomic = "64"))]
    #[test]
    fn atomic_page_table_layout_matches_page_table() {
        assert_eq!(
            core::mem::size_of::<AtomicPageTableEntry>(),
            core::mem::size_of::<PageTableEntry>()
        );
        assert_eq!(
            core::mem::align_of::<AtomicPageTableEntry>(),
            core::mem::align_of::<PageTableEntry>()
        );
        assert_eq!(
            core::mem::size_of::<AtomicPageTable>(),
            core::mem::size_of::<PageTable>()
        );
        assert_eq!(
            core::mem::align_of::<AtomicPageTable>(),
            core::mem::align_of::<PageTable>()
        );
    }
}

#[cfg(all(test, feature = "step_trait"))]
mod tests {
    use super::*;
    use core::iter::Step;

    #[test]
    fn page_table_index_step_overflowing() {
        assert_eq!(
            Step::forward_overflowing(PageTableIndex::new(0), 511),
            (PageTableIndex::new(511), false)
        );
        assert_eq!(
            Step::backward_overflowing(PageTableIndex::new(511), 511),
            (PageTableIndex::new(0), false)
        );

        // Overflow the 0..512 range: only the flag is specified.
        assert!(Step::forward_overflowing(PageTableIndex::new(511), 1).1);
        assert!(Step::backward_overflowing(PageTableIndex::new(0), 1).1);
    }
}
