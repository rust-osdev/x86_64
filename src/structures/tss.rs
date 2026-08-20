//! Provides a type for the task state segment structure.

use core::{
    fmt::{self, Display},
    mem::size_of,
};

use crate::addr::{DefaultVirtAddrValidity, VirtAddrGeneric, VirtAddrValidity};

/// In 64-bit mode the TSS holds information that is not
/// directly related to the task-switch mechanism,
/// but is used for stack switching when an interrupt or exception occurs.
#[repr(C, packed(4))]
pub struct TaskStateSegment<V: VirtAddrValidity = DefaultVirtAddrValidity> {
    reserved_1: u32,
    /// The full 64-bit canonical forms of the stack pointers (RSP) for privilege levels 0-2.
    /// The stack pointers used when a privilege level change occurs from a lower privilege level to a higher one.
    pub privilege_stack_table: [VirtAddrGeneric<V>; 3],
    reserved_2: u64,
    /// The full 64-bit canonical forms of the interrupt stack table (IST) pointers.
    /// The stack pointers used when an entry in the Interrupt Descriptor Table has an IST value other than 0.
    pub interrupt_stack_table: [VirtAddrGeneric<V>; 7],
    reserved_3: u64,
    reserved_4: u16,
    /// The 16-bit offset to the I/O permission bit map from the 64-bit TSS base. It must not
    /// exceed `0xDFFF`.
    pub iomap_base: u16,
}

impl<V: VirtAddrValidity> TaskStateSegment<V> {
    /// Creates a new TSS with zeroed privilege and interrupt stack table and an
    /// empty I/O-Permission Bitmap.
    ///
    /// As we always set the TSS segment limit to
    /// `size_of::<TaskStateSegment>() - 1`, this means that `iomap_base` is
    /// initialized to `size_of::<TaskStateSegment>()`.
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub fn new_with_validity() -> Self {
        TaskStateSegment {
            privilege_stack_table: [VirtAddrGeneric::zero(); 3],
            interrupt_stack_table: [VirtAddrGeneric::zero(); 7],
            iomap_base: size_of::<Self>() as u16,
            reserved_1: 0,
            reserved_2: 0,
            reserved_3: 0,
            reserved_4: 0,
        }
    }
}

// These traits are implemented manually because Rust 1.59 has limited derive support for generic
// packed structs. They can use derive once the MSRV is raised to Rust 1.69.
impl<V: VirtAddrValidity> Copy for TaskStateSegment<V> {}

impl<V: VirtAddrValidity> Clone for TaskStateSegment<V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<V: VirtAddrValidity> fmt::Debug for TaskStateSegment<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reserved_1 = self.reserved_1;
        let privilege_stack_table = self.privilege_stack_table;
        let reserved_2 = self.reserved_2;
        let interrupt_stack_table = self.interrupt_stack_table;
        let reserved_3 = self.reserved_3;
        let reserved_4 = self.reserved_4;
        let iomap_base = self.iomap_base;

        f.debug_struct("TaskStateSegment")
            .field("reserved_1", &reserved_1)
            .field("privilege_stack_table", &privilege_stack_table)
            .field("reserved_2", &reserved_2)
            .field("interrupt_stack_table", &interrupt_stack_table)
            .field("reserved_3", &reserved_3)
            .field("reserved_4", &reserved_4)
            .field("iomap_base", &iomap_base)
            .finish()
    }
}

impl TaskStateSegment<DefaultVirtAddrValidity> {
    /// Creates a new TSS with the default virtual-address validity.
    ///
    /// Stack addresses assigned later retain their creation-time validity guarantees.
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub fn new() -> Self {
        Self::new_with_validity()
    }
}

impl<V: VirtAddrValidity> Default for TaskStateSegment<V> {
    #[inline]
    fn default() -> Self {
        Self::new_with_validity()
    }
}

/// The given IO permissions bitmap is invalid.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InvalidIoMap {
    /// The IO permissions bitmap is before the TSS. It must be located after the TSS.
    IoMapBeforeTss,
    /// The IO permissions bitmap is too far from the TSS. It must be within `0xdfff` bytes of the
    /// start of the TSS. Note that if the IO permissions bitmap is located before the TSS, then
    /// `IoMapBeforeTss` will be returned instead.
    TooFarFromTss {
        /// The distance of the IO permissions bitmap from the beginning of the TSS.
        distance: usize,
    },
    /// The final byte of the IO permissions bitmap was not 0xff
    InvalidTerminatingByte {
        /// The byte found at the end of the IO permissions bitmap.
        byte: u8,
    },
    /// The IO permissions bitmap exceeds the maximum length (8193).
    TooLong {
        /// The length of the IO permissions bitmap.
        len: usize,
    },
    /// The `iomap_base` in the `TaskStateSegment` struct was not what was expected.
    InvalidBase {
        /// The expected `iomap_base` to be set in the `TaskStateSegment` struct.
        expected: u16,
        /// The actual `iomap_base` set in the `TaskStateSegment` struct.
        got: u16,
    },
}

impl Display for InvalidIoMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            InvalidIoMap::IoMapBeforeTss => {
                write!(f, "the IO permissions bitmap is before the TSS")
            }
            InvalidIoMap::TooFarFromTss { distance } => write!(
                f,
                "the IO permissions bitmap is too far from the TSS (distance {distance})"
            ),
            InvalidIoMap::InvalidTerminatingByte { byte } => write!(
                f,
                "The final byte of the IO permissions bitmap was not 0xff ({byte}"
            ),
            InvalidIoMap::TooLong { len } => {
                write!(
                    f,
                    "The IO permissions bitmap exceeds the maximum length ({len} > 8193)"
                )
            }
            InvalidIoMap::InvalidBase { expected, got } => write!(
                f,
                "the `iomap_base` in the `TaskStateSegment` struct was not what was expected (expected {expected}, got {got})"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn check_tss_size() {
        // Per the SDM, the minimum size of a TSS is 0x68 bytes, giving a
        // minimum limit of 0x67.
        assert_eq!(size_of::<TaskStateSegment>(), 0x68);
        #[cfg(feature = "virt_addr_57")]
        assert_eq!(
            size_of::<TaskStateSegment<crate::addr::FixedValidity<57>>>(),
            0x68
        );
        #[cfg(feature = "virt_addr_rt")]
        assert_eq!(
            size_of::<TaskStateSegment<crate::addr::RuntimeValidity>>(),
            0x68
        );
    }
}
