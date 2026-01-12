//! Types for the Global Descriptor Table and segment selectors.

pub use crate::registers::segmentation::SegmentSelector;
use crate::structures::tss::{InvalidIoMap, TaskStateSegment};
use crate::PrivilegeLevel;
use bit_field::BitField;
use bitflags::bitflags;
use core::{cmp, fmt, mem};
// imports for intra-doc links
#[cfg(doc)]
use crate::registers::segmentation::{Segment, CS, SS};

#[cfg(all(feature = "instructions", target_arch = "x86_64"))]
use core::sync::atomic::{AtomicU64 as EntryValue, Ordering};
#[cfg(not(all(feature = "instructions", target_arch = "x86_64")))]
use u64 as EntryValue;

/// 8-byte entry in a descriptor table.
///
/// A [`GlobalDescriptorTable`] (or LDT) is an array of these entries, and
/// [`SegmentSelector`]s index into this array. Each [`Descriptor`] in the table
/// uses either 1 Entry (if it is a [`UserSegment`](Descriptor::UserSegment)) or
/// 2 Entries (if it is a [`SystemSegment`](Descriptor::SystemSegment)). This
/// type exists to give users access to the raw entry bits in a GDT.
#[repr(transparent)]
pub struct Entry(EntryValue);

impl Entry {
    // Create a new Entry from a raw value.
    const fn new(raw: u64) -> Self {
        #[cfg(all(feature = "instructions", target_arch = "x86_64"))]
        let raw = EntryValue::new(raw);
        Self(raw)
    }

    /// The raw bits for this entry. Depending on the [`Descriptor`] type, these
    /// bits may correspond to those in [`DescriptorFlags`].
    pub fn raw(&self) -> u64 {
        // TODO: Make this const fn when AtomicU64::load is const.
        #[cfg(all(feature = "instructions", target_arch = "x86_64"))]
        let raw = self.0.load(Ordering::SeqCst);
        #[cfg(not(all(feature = "instructions", target_arch = "x86_64")))]
        let raw = self.0;
        raw
    }
}

impl Clone for Entry {
    fn clone(&self) -> Self {
        Self::new(self.raw())
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.raw() == other.raw()
    }
}

impl Eq for Entry {}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display inner value as hex
        write!(f, "Entry({:#018x})", self.raw())
    }
}

/// A 64-bit mode global descriptor table (GDT).
///
/// In 64-bit mode, segmentation is not supported. The GDT is used nonetheless, for example for
/// switching between user and kernel mode or for loading a TSS.
///
/// The GDT has a fixed maximum size given by the `MAX` const generic parameter.
/// Overflowing this limit by adding too many [`Descriptor`]s via
/// [`GlobalDescriptorTable::append`] will panic.
///
/// You do **not** need to add a null segment descriptor yourself - this is already done
/// internally. This means you can add up to `MAX - 1` additional [`Entry`]s to
/// this table. Note that some [`Descriptor`]s may take up 2 [`Entry`]s.
///
/// Data segment registers in ring 0 can be loaded with the null segment selector. When running in
/// ring 3, the `ss` register must point to a valid data segment which can be obtained through the
/// [`Descriptor::user_data_segment()`](Descriptor::user_data_segment) function. Code segments must
/// be valid and non-null at all times and can be obtained through the
/// [`Descriptor::kernel_code_segment()`](Descriptor::kernel_code_segment) and
/// [`Descriptor::user_code_segment()`](Descriptor::user_code_segment) in rings 0 and 3
/// respectively.
///
/// For more info, see:
/// [x86 Instruction Reference for `mov`](https://www.felixcloutier.com/x86/mov#64-bit-mode-exceptions),
/// [Intel Manual](https://software.intel.com/sites/default/files/managed/39/c5/325462-sdm-vol-1-2abcd-3abcd.pdf),
/// [AMD Manual](https://www.amd.com/system/files/TechDocs/24593.pdf)
///
/// # Example
/// ```
/// use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor};
///
/// let mut gdt = GlobalDescriptorTable::new();
/// gdt.append(Descriptor::kernel_code_segment());
/// gdt.append(Descriptor::user_code_segment());
/// gdt.append(Descriptor::user_data_segment());
///
/// // Add entry for TSS, call gdt.load() then update segment registers
/// ```

#[derive(Debug, Clone)]
pub struct GlobalDescriptorTable<const MAX: usize = 8> {
    table: [Entry; MAX],
    len: usize,
}

impl GlobalDescriptorTable {
    /// Creates an empty GDT with the default length of 8.
    pub const fn new() -> Self {
        Self::empty()
    }
}

impl Default for GlobalDescriptorTable {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const MAX: usize> GlobalDescriptorTable<MAX> {
    /// Creates an empty GDT which can hold `MAX` number of [`Entry`]s.
    #[inline]
    pub const fn empty() -> Self {
        // TODO: Replace with compiler error when feature(generic_const_exprs) is stable.
        assert!(MAX > 0, "A GDT cannot have 0 entries");
        assert!(MAX <= (1 << 13), "A GDT can only have at most 2^13 entries");

        // TODO: Replace with inline_const when it's stable.
        #[allow(clippy::declare_interior_mutable_const)]
        const NULL: Entry = Entry::new(0);
        Self {
            table: [NULL; MAX],
            len: 1,
        }
    }

    /// Forms a GDT from a slice of `u64`.
    ///
    /// This method allows for creation of a GDT with malformed or invalid
    /// entries. However, it is safe because loading a GDT with invalid
    /// entires doesn't do anything until those entries are used. For example,
    /// [`CS::set_reg`] and [`load_tss`](crate::instructions::tables::load_tss)
    /// are both unsafe for this reason.
    ///
    /// Panics if:
    /// * the provided slice has more than `MAX` entries
    /// * the provided slice is empty
    /// * the first entry is not zero
    #[cfg_attr(
        not(all(feature = "instructions", target_arch = "x86_64")),
        allow(rustdoc::broken_intra_doc_links)
    )]
    #[inline]
    pub const fn from_raw_entries(slice: &[u64]) -> Self {
        let len = slice.len();
        let mut table = Self::empty().table;
        let mut idx = 0;

        assert!(len > 0, "cannot initialize GDT with empty slice");
        assert!(slice[0] == 0, "first GDT entry must be zero");
        assert!(
            len <= MAX,
            "cannot initialize GDT with slice exceeding the maximum length"
        );

        while idx < len {
            table[idx] = Entry::new(slice[idx]);
            idx += 1;
        }

        Self { table, len }
    }

    /// Get a reference to the internal [`Entry`] table.
    ///
    /// The resulting slice may contain system descriptors, which span two [`Entry`]s.
    #[inline]
    pub fn entries(&self) -> &[Entry] {
        &self.table[..self.len]
    }

    /// Appends the given segment descriptor to the GDT, returning the segment selector.
    ///
    /// Note that depending on the type of the [`Descriptor`] this may append
    /// either one or two new [`Entry`]s to the table.
    ///
    /// Panics if the GDT doesn't have enough free entries.
    #[inline]
    #[rustversion::attr(since(1.83), const)]
    pub fn append(&mut self, entry: Descriptor) -> SegmentSelector {
        let index = match entry {
            Descriptor::UserSegment(value) => {
                if self.len > self.table.len().saturating_sub(1) {
                    panic!("GDT full")
                }
                self.push(value)
            }
            Descriptor::SystemSegment(value_low, value_high) => {
                if self.len > self.table.len().saturating_sub(2) {
                    panic!("GDT requires two free spaces to hold a SystemSegment")
                }
                let index = self.push(value_low);
                self.push(value_high);
                index
            }
        };
        SegmentSelector::new(index as u16, entry.dpl())
    }

    /// Loads the GDT in the CPU using the `lgdt` instruction. This does **not** alter any of the
    /// segment registers; you **must** (re)load them yourself using [the appropriate
    /// functions](crate::instructions::segmentation):
    /// [`SS::set_reg()`] and [`CS::set_reg()`].
    #[cfg(all(feature = "instructions", target_arch = "x86_64"))]
    #[inline]
    pub fn load(&'static self) {
        // SAFETY: static lifetime ensures no modification after loading.
        unsafe { self.load_unsafe() };
    }

    /// Loads the GDT in the CPU using the `lgdt` instruction. This does **not** alter any of the
    /// segment registers; you **must** (re)load them yourself using [the appropriate
    /// functions](crate::instructions::segmentation):
    /// [`SS::set_reg()`] and [`CS::set_reg()`].
    ///
    /// # Safety
    ///
    /// Unlike `load` this function will not impose a static lifetime constraint
    /// this means its up to the user to ensure that there will be no modifications
    /// after loading and that the GDT will live for as long as it's loaded.
    ///
    #[cfg(all(feature = "instructions", target_arch = "x86_64"))]
    #[inline]
    pub unsafe fn load_unsafe(&self) {
        use crate::instructions::tables::lgdt;
        unsafe {
            lgdt(&self.pointer());
        }
    }

    #[inline]
    #[rustversion::attr(since(1.83), const)]
    fn push(&mut self, value: u64) -> usize {
        let index = self.len;
        self.table[index] = Entry::new(value);
        self.len += 1;
        index
    }

    /// Returns the value of the limit for a gdt pointer. It is one less than the number of bytes of the table.
    pub const fn limit(&self) -> u16 {
        use core::mem::size_of;
        // 0 < self.next_free <= MAX <= 2^13, so the limit calculation
        // will not underflow or overflow.
        (self.len * size_of::<u64>() - 1) as u16
    }

    /// Creates the descriptor pointer for this table. This pointer can only be
    /// safely used if the table is never modified or destroyed while in use.
    #[cfg(all(feature = "instructions", target_arch = "x86_64"))]
    fn pointer(&self) -> super::DescriptorTablePointer {
        super::DescriptorTablePointer {
            base: crate::VirtAddr::new(self.table.as_ptr() as u64),
            limit: self.limit(),
        }
    }
}

/// A 64-bit mode segment descriptor.
///
/// Segmentation is no longer supported in 64-bit mode, so most of the descriptor
/// contents are ignored.
#[derive(Debug, Clone, Copy)]
pub enum Descriptor {
    /// Descriptor for a code or data segment.
    ///
    /// Since segmentation is no longer supported in 64-bit mode, almost all of
    /// code and data descriptors is ignored. Only some flags are still used.
    UserSegment(u64),
    /// A system segment descriptor such as a LDT or TSS descriptor.
    SystemSegment(u64, u64),
}

bitflags! {
    /// Flags for a GDT descriptor. Not all flags are valid for all descriptor types.
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct DescriptorFlags: u64 {
        /// Set by the processor if this segment has been accessed. Only cleared by software.
        /// _Setting_ this bit in software prevents GDT writes on first use.
        const ACCESSED          = 1 << 40;
        /// For 32-bit data segments, sets the segment as writable. For 32-bit code segments,
        /// sets the segment as _readable_. In 64-bit mode, ignored for all segments.
        const WRITABLE          = 1 << 41;
        /// For code segments, sets the segment as “conforming”, influencing the
        /// privilege checks that occur on control transfers. For 32-bit data segments,
        /// sets the segment as "expand down". In 64-bit mode, ignored for data segments.
        const CONFORMING        = 1 << 42;
        /// This flag must be set for code segments and unset for data segments.
        const EXECUTABLE        = 1 << 43;
        /// This flag must be set for user segments (in contrast to system segments).
        const USER_SEGMENT      = 1 << 44;
        /// These two bits encode the Descriptor Privilege Level (DPL) for this descriptor.
        /// If both bits are set, the DPL is Ring 3, if both are unset, the DPL is Ring 0.
        const DPL_RING_3        = 3 << 45;
        /// Must be set for any segment, causes a segment not present exception if not set.
        const PRESENT           = 1 << 47;
        /// Available for use by the Operating System
        const AVAILABLE         = 1 << 52;
        /// Must be set for 64-bit code segments, unset otherwise.
        const LONG_MODE         = 1 << 53;
        /// Use 32-bit (as opposed to 16-bit) operands. If [`LONG_MODE`][Self::LONG_MODE] is set,
        /// this must be unset. In 64-bit mode, ignored for data segments.
        const DEFAULT_SIZE      = 1 << 54;
        /// Limit field is scaled by 4096 bytes. In 64-bit mode, ignored for all segments.
        const GRANULARITY       = 1 << 55;

        /// Bits `0..=15` of the limit field (ignored in 64-bit mode)
        const LIMIT_0_15        = 0xFFFF;
        /// Bits `16..=19` of the limit field (ignored in 64-bit mode)
        const LIMIT_16_19       = 0xF << 48;
        /// Bits `0..=23` of the base field (ignored in 64-bit mode, except for fs and gs)
        const BASE_0_23         = 0xFF_FFFF << 16;
        /// Bits `24..=31` of the base field (ignored in 64-bit mode, except for fs and gs)
        const BASE_24_31        = 0xFF << 56;
    }
}

/// The following constants define default values for common GDT entries. They
/// are all "flat" segments, meaning they can access the entire address space.
/// These values all set [`WRITABLE`][DescriptorFlags::WRITABLE] and
/// [`ACCESSED`][DescriptorFlags::ACCESSED]. They also match the values loaded
/// by the `syscall`/`sysret` and `sysenter`/`sysexit` instructions.
///
/// In short, these values disable segmentation, permission checks, and access
/// tracking at the GDT level. Kernels using these values should use paging to
/// implement this functionality.
impl DescriptorFlags {
    // Flags that we set for all our default segments
    const COMMON: Self = Self::from_bits_truncate(
        Self::USER_SEGMENT.bits()
            | Self::PRESENT.bits()
            | Self::WRITABLE.bits()
            | Self::ACCESSED.bits()
            | Self::LIMIT_0_15.bits()
            | Self::LIMIT_16_19.bits()
            | Self::GRANULARITY.bits(),
    );
    /// A kernel data segment (64-bit or flat 32-bit)
    pub const KERNEL_DATA: Self =
        Self::from_bits_truncate(Self::COMMON.bits() | Self::DEFAULT_SIZE.bits());
    /// A flat 32-bit kernel code segment
    pub const KERNEL_CODE32: Self = Self::from_bits_truncate(
        Self::COMMON.bits() | Self::EXECUTABLE.bits() | Self::DEFAULT_SIZE.bits(),
    );
    /// A 64-bit kernel code segment
    pub const KERNEL_CODE64: Self = Self::from_bits_truncate(
        Self::COMMON.bits() | Self::EXECUTABLE.bits() | Self::LONG_MODE.bits(),
    );
    /// A user data segment (64-bit or flat 32-bit)
    pub const USER_DATA: Self =
        Self::from_bits_truncate(Self::KERNEL_DATA.bits() | Self::DPL_RING_3.bits());
    /// A flat 32-bit user code segment
    pub const USER_CODE32: Self =
        Self::from_bits_truncate(Self::KERNEL_CODE32.bits() | Self::DPL_RING_3.bits());
    /// A 64-bit user code segment
    pub const USER_CODE64: Self =
        Self::from_bits_truncate(Self::KERNEL_CODE64.bits() | Self::DPL_RING_3.bits());
}

impl Descriptor {
    /// Returns the Descriptor Privilege Level (DPL). When using this descriptor
    /// via a [`SegmentSelector`], the RPL and Current Privilege Level (CPL)
    /// must less than or equal to the DPL, except for stack segments where the
    /// RPL, CPL, and DPL must all be equal.
    #[inline]
    pub const fn dpl(self) -> PrivilegeLevel {
        let value_low = match self {
            Descriptor::UserSegment(v) => v,
            Descriptor::SystemSegment(v, _) => v,
        };
        let dpl = (value_low & DescriptorFlags::DPL_RING_3.bits()) >> 45;
        PrivilegeLevel::from_u16(dpl as u16)
    }

    /// Creates a segment descriptor for a 64-bit kernel code segment. Suitable
    /// for use with `syscall` or 64-bit `sysenter`.
    #[inline]
    pub const fn kernel_code_segment() -> Descriptor {
        Descriptor::UserSegment(DescriptorFlags::KERNEL_CODE64.bits())
    }

    /// Creates a segment descriptor for a kernel data segment (32-bit or
    /// 64-bit). Suitable for use with `syscall` or `sysenter`.
    #[inline]
    pub const fn kernel_data_segment() -> Descriptor {
        Descriptor::UserSegment(DescriptorFlags::KERNEL_DATA.bits())
    }

    /// Creates a segment descriptor for a ring 3 data segment (32-bit or
    /// 64-bit). Suitable for use with `sysret` or `sysexit`.
    #[inline]
    pub const fn user_data_segment() -> Descriptor {
        Descriptor::UserSegment(DescriptorFlags::USER_DATA.bits())
    }

    /// Creates a segment descriptor for a 64-bit ring 3 code segment. Suitable
    /// for use with `sysret` or `sysexit`.
    #[inline]
    pub const fn user_code_segment() -> Descriptor {
        Descriptor::UserSegment(DescriptorFlags::USER_CODE64.bits())
    }

    /// Creates a TSS system descriptor for the given TSS.
    ///
    /// While it is possible to create multiple Descriptors that point to the
    /// same TSS, this generally isn't recommended, as the TSS usually contains
    /// per-CPU information such as the RSP and IST pointers. Instead, there
    /// should be exactly one TSS and one corresponding TSS Descriptor per CPU.
    /// Then, each of these descriptors should be placed in a GDT (which can
    /// either be global or per-CPU).
    #[inline]
    pub fn tss_segment(tss: &'static TaskStateSegment) -> Descriptor {
        // SAFETY: The pointer is derived from a &'static reference, which ensures its validity.
        unsafe { Self::tss_segment_unchecked(tss) }
    }

    /// Similar to [`Descriptor::tss_segment`], but unsafe since it does not enforce a lifetime
    /// constraint on the provided TSS.
    ///
    /// # Safety
    /// The caller must ensure that the passed pointer is valid for as long as the descriptor is
    /// being used.
    #[inline]
    pub unsafe fn tss_segment_unchecked(tss: *const TaskStateSegment) -> Descriptor {
        // SAFETY: if iomap_size is zero, there are no requirements to uphold.
        unsafe { Self::tss_segment_raw(tss, 0) }
    }

    /// Creates a TSS system descriptor for the given TSS, setting up the IO permissions bitmap.
    ///
    /// # Example
    ///
    /// ```
    /// use x86_64::structures::gdt::Descriptor;
    /// use x86_64::structures::tss::TaskStateSegment;
    ///
    /// /// A helper that places some I/O map bytes behind a TSS.
    /// #[repr(C)]
    /// struct TssWithIOMap {
    ///     tss: TaskStateSegment,
    ///     iomap: [u8; 5],
    /// }
    ///
    /// static TSS: TssWithIOMap = TssWithIOMap {
    ///     tss: TaskStateSegment::new(),
    ///     iomap: [0xff, 0xff, 0x00, 0x80, 0xff],
    /// };
    ///
    /// let tss = Descriptor::tss_segment_with_iomap(&TSS.tss, &TSS.iomap).unwrap();
    /// ```
    pub fn tss_segment_with_iomap(
        tss: &'static TaskStateSegment,
        iomap: &'static [u8],
    ) -> Result<Descriptor, InvalidIoMap> {
        if iomap.len() > 8193 {
            return Err(InvalidIoMap::TooLong { len: iomap.len() });
        }

        let iomap_addr = iomap.as_ptr() as usize;
        let tss_addr = tss as *const _ as usize;

        if tss_addr > iomap_addr {
            return Err(InvalidIoMap::IoMapBeforeTss);
        }

        let base = iomap_addr - tss_addr;
        if base > 0xdfff {
            return Err(InvalidIoMap::TooFarFromTss { distance: base });
        }

        let last_byte = *iomap.last().unwrap_or(&0xff);
        if last_byte != 0xff {
            return Err(InvalidIoMap::InvalidTerminatingByte { byte: last_byte });
        }

        if tss.iomap_base != base as u16 {
            return Err(InvalidIoMap::InvalidBase {
                expected: base as u16,
                got: tss.iomap_base,
            });
        }

        // SAFETY: all invariants checked above
        Ok(unsafe { Self::tss_segment_raw(tss, iomap.len() as u16) })
    }

    /// Creates a TSS system descriptor for the given TSS, setting up the IO permissions bitmap.
    ///
    /// # Safety
    ///
    /// There must be a valid IO map at `(tss as *const u8).offset(tss.iomap_base)`
    /// of length `iomap_size`, with the terminating `0xFF` byte. Additionally, `iomap_base` must
    /// not exceed `0xDFFF`.
    unsafe fn tss_segment_raw(tss: *const TaskStateSegment, iomap_size: u16) -> Descriptor {
        use self::DescriptorFlags as Flags;

        let ptr = tss as u64;

        let mut low = Flags::PRESENT.bits();
        // base
        low.set_bits(16..40, ptr.get_bits(0..24));
        low.set_bits(56..64, ptr.get_bits(24..32));
        // limit (the `-1` is needed since the bound is inclusive)
        let iomap_limit = u64::from(unsafe { (*tss).iomap_base }) + u64::from(iomap_size);
        low.set_bits(
            0..16,
            cmp::max(mem::size_of::<TaskStateSegment>() as u64, iomap_limit) - 1,
        );
        // type (0b1001 = available 64-bit tss)
        low.set_bits(40..44, 0b1001);

        let mut high = 0;
        high.set_bits(0..32, ptr.get_bits(32..64));

        Descriptor::SystemSegment(low, high)
    }
}

#[cfg(test)]
mod tests {
    use super::DescriptorFlags as Flags;
    use super::*;

    #[test]
    #[rustfmt::skip]
    pub fn linux_kernel_defaults() {
        // Make sure our defaults match the ones used by the Linux kernel.
        // Constants pulled from an old version of arch/x86/kernel/cpu/common.c
        assert_eq!(Flags::KERNEL_CODE64.bits(), 0x00af9b000000ffff);
        assert_eq!(Flags::KERNEL_CODE32.bits(), 0x00cf9b000000ffff);
        assert_eq!(Flags::KERNEL_DATA.bits(),   0x00cf93000000ffff);
        assert_eq!(Flags::USER_CODE64.bits(),   0x00affb000000ffff);
        assert_eq!(Flags::USER_CODE32.bits(),   0x00cffb000000ffff);
        assert_eq!(Flags::USER_DATA.bits(),     0x00cff3000000ffff);
    }

    // Makes a GDT that has two free slots
    fn make_six_entry_gdt() -> GlobalDescriptorTable {
        let mut gdt = GlobalDescriptorTable::new();
        gdt.append(Descriptor::kernel_code_segment());
        gdt.append(Descriptor::kernel_data_segment());
        gdt.append(Descriptor::UserSegment(DescriptorFlags::USER_CODE32.bits()));
        gdt.append(Descriptor::user_data_segment());
        gdt.append(Descriptor::user_code_segment());
        assert_eq!(gdt.len, 6);
        gdt
    }

    static TSS: TaskStateSegment = TaskStateSegment::new();

    fn make_full_gdt() -> GlobalDescriptorTable {
        let mut gdt = make_six_entry_gdt();
        gdt.append(Descriptor::tss_segment(&TSS));
        assert_eq!(gdt.len, 8);
        gdt
    }

    #[test]
    pub fn push_max_segments() {
        // Make sure we don't panic with user segments
        let mut gdt = make_six_entry_gdt();
        gdt.append(Descriptor::user_data_segment());
        assert_eq!(gdt.len, 7);
        gdt.append(Descriptor::user_data_segment());
        assert_eq!(gdt.len, 8);
        // Make sure we don't panic with system segments
        let _ = make_full_gdt();
    }

    #[test]
    #[should_panic]
    pub fn panic_user_segment() {
        let mut gdt = make_full_gdt();
        gdt.append(Descriptor::user_data_segment());
    }

    #[test]
    #[should_panic]
    pub fn panic_system_segment() {
        let mut gdt = make_six_entry_gdt();
        gdt.append(Descriptor::user_data_segment());
        // We have one free slot, but the GDT requires two
        gdt.append(Descriptor::tss_segment(&TSS));
    }

    #[test]
    pub fn from_entries() {
        let raw = [0, Flags::KERNEL_CODE64.bits(), Flags::KERNEL_DATA.bits()];
        let gdt = GlobalDescriptorTable::<3>::from_raw_entries(&raw);
        assert_eq!(gdt.table.len(), 3);
        assert_eq!(gdt.entries().len(), 3);
    }

    #[test]
    pub fn descriptor_dpl() {
        assert_eq!(
            Descriptor::kernel_code_segment().dpl(),
            PrivilegeLevel::Ring0
        );
        assert_eq!(
            Descriptor::kernel_data_segment().dpl(),
            PrivilegeLevel::Ring0
        );
        assert_eq!(Descriptor::user_code_segment().dpl(), PrivilegeLevel::Ring3);
        assert_eq!(Descriptor::user_code_segment().dpl(), PrivilegeLevel::Ring3);
    }
}
