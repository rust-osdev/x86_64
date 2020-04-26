//! Types for the Global Descriptor Table and segment selectors.

use crate::structures::tss::TaskStateSegment;
use crate::PrivilegeLevel;
use bit_field::BitField;
use bitflags::bitflags;
use core::fmt;

/// Specifies which element to load into a segment from
/// descriptor tables (i.e., is a index to LDT or GDT table
/// with some additional flags).
///
/// See Intel 3a, Section 3.4.2 "Segment Selectors"
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    /// Creates a new SegmentSelector
    ///
    /// # Arguments
    ///  * `index`: index in GDT or LDT array (not the offset)
    ///  * `rpl`: the requested privilege level
    #[inline]
    pub const fn new(index: u16, rpl: PrivilegeLevel) -> SegmentSelector {
        SegmentSelector(index << 3 | (rpl as u16))
    }

    /// Returns the GDT index.
    #[inline]
    pub fn index(self) -> u16 {
        self.0 >> 3
    }

    /// Returns the requested privilege level.
    #[inline]
    pub fn rpl(self) -> PrivilegeLevel {
        PrivilegeLevel::from_u16(self.0.get_bits(0..2))
    }

    /// Set the privilege level for this Segment selector.
    #[inline]
    pub fn set_rpl(&mut self, rpl: PrivilegeLevel) {
        self.0.set_bits(0..2, rpl as u16);
    }
}

impl fmt::Debug for SegmentSelector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = f.debug_struct("SegmentSelector");
        s.field("index", &self.index());
        s.field("rpl", &self.rpl());
        s.finish()
    }
}

/// A 64-bit mode global descriptor table (GDT).
///
/// In 64-bit mode, segmentation is not supported. The GDT is used nonetheless, for example for
/// switching between user and kernel mode or for loading a TSS.
///
/// The GDT has a fixed size of 8 entries, trying to add more entries will panic.
///
/// You do **not** need to add a null segment descriptor yourself - this is already done
/// internally.
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
/// gdt.add_entry(Descriptor::kernel_code_segment());
/// gdt.add_entry(Descriptor::user_code_segment());
/// gdt.add_entry(Descriptor::user_data_segment());
///
/// // Add entry for TSS, call gdt.load() then update segment registers
/// ```

#[derive(Debug, Clone)]
pub struct GlobalDescriptorTable {
    table: [u64; 8],
    next_free: usize,
}

impl GlobalDescriptorTable {
    /// Creates an empty GDT.
    #[inline]
    pub const fn new() -> GlobalDescriptorTable {
        GlobalDescriptorTable {
            table: [0; 8],
            next_free: 1,
        }
    }

    /// Adds the given segment descriptor to the GDT, returning the segment selector.
    ///
    /// Panics if the GDT has no free entries left.
    #[inline]
    pub fn add_entry(&mut self, entry: Descriptor) -> SegmentSelector {
        let index = match entry {
            Descriptor::UserSegment(value) => self.push(value),
            Descriptor::SystemSegment(value_low, value_high) => {
                let index = self.push(value_low);
                self.push(value_high);
                index
            }
        };

        let rpl = match entry {
            Descriptor::UserSegment(value) => {
                if DescriptorFlags::from_bits_truncate(value).contains(DescriptorFlags::DPL_RING_3)
                {
                    PrivilegeLevel::Ring3
                } else {
                    PrivilegeLevel::Ring0
                }
            }
            Descriptor::SystemSegment(_, _) => PrivilegeLevel::Ring0,
        };

        SegmentSelector::new(index as u16, rpl)
    }

    /// Loads the GDT in the CPU using the `lgdt` instruction. This does **not** alter any of the
    /// segment registers; you **must** (re)load them yourself using [the appropriate
    /// functions](crate::instructions::segmentation):
    /// [load_ss](crate::instructions::segmentation::load_ss),
    /// [set_cs](crate::instructions::segmentation::set_cs).
    #[cfg(target_arch = "x86_64")]
    #[inline]
    pub fn load(&'static self) {
        use crate::instructions::tables::{lgdt, DescriptorTablePointer};
        use core::mem::size_of;

        let ptr = DescriptorTablePointer {
            base: self.table.as_ptr() as u64,
            limit: (self.table.len() * size_of::<u64>() - 1) as u16,
        };

        unsafe { lgdt(&ptr) };
    }

    #[inline]
    fn push(&mut self, value: u64) -> usize {
        if self.next_free < self.table.len() {
            let index = self.next_free;
            self.table[index] = value;
            self.next_free += 1;
            index
        } else {
            panic!("GDT full");
        }
    }
}

/// A 64-bit mode segment descriptor.
///
/// Segmentation is no longer supported in 64-bit mode, so most of the descriptor
/// contents are ignored.
#[derive(Debug, Clone)]
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
    pub struct DescriptorFlags: u64 {
        /// For data segments, this flag sets the segment as writable. Ignored for code segments.
        const WRITABLE          = 1 << 41;
        /// Marks a code segment as “conforming”. This influences the privilege checks that
        /// occur on control transfers.
        const CONFORMING        = 1 << 42;
        /// This flag must be set for code segments.
        const EXECUTABLE        = 1 << 43;
        /// This flag must be set for user segments (in contrast to system segments).
        const USER_SEGMENT      = 1 << 44;
        /// Must be set for any segment, causes a segment not present exception if not set.
        const PRESENT           = 1 << 47;
        /// Must be set for long mode code segments.
        const LONG_MODE         = 1 << 53;

        /// The DPL for this descriptor is Ring 3
        const DPL_RING_3        = 3 << 45;
    }
}

impl Descriptor {
    /// Creates a segment descriptor for a long mode kernel code segment.
    #[inline]
    pub fn kernel_code_segment() -> Descriptor {
        use self::DescriptorFlags as Flags;

        let flags = Flags::USER_SEGMENT | Flags::PRESENT | Flags::EXECUTABLE | Flags::LONG_MODE;
        Descriptor::UserSegment(flags.bits())
    }

    /// Creates a segment descriptor for a long mode ring 3 data segment.
    #[inline]
    pub fn user_data_segment() -> Descriptor {
        use self::DescriptorFlags as Flags;

        let flags = Flags::USER_SEGMENT | Flags::PRESENT | Flags::WRITABLE | Flags::DPL_RING_3;
        Descriptor::UserSegment(flags.bits())
    }

    /// Creates a segment descriptor for a long mode ring 3 code segment.
    #[inline]
    pub fn user_code_segment() -> Descriptor {
        use self::DescriptorFlags as Flags;

        let flags = Flags::USER_SEGMENT
            | Flags::PRESENT
            | Flags::EXECUTABLE
            | Flags::LONG_MODE
            | Flags::DPL_RING_3;
        Descriptor::UserSegment(flags.bits())
    }

    /// Creates a TSS system descriptor for the given TSS.
    #[inline]
    pub fn tss_segment(tss: &'static TaskStateSegment) -> Descriptor {
        use self::DescriptorFlags as Flags;
        use core::mem::size_of;

        let ptr = tss as *const _ as u64;

        let mut low = Flags::PRESENT.bits();
        // base
        low.set_bits(16..40, ptr.get_bits(0..24));
        low.set_bits(56..64, ptr.get_bits(24..32));
        // limit (the `-1` in needed since the bound is inclusive)
        low.set_bits(0..16, (size_of::<TaskStateSegment>() - 1) as u64);
        // type (0b1001 = available 64-bit tss)
        low.set_bits(40..44, 0b1001);

        let mut high = 0;
        high.set_bits(0..32, ptr.get_bits(32..64));

        Descriptor::SystemSegment(low, high)
    }
}
