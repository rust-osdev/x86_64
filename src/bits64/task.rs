//! Helpers to program the task state segment.
//! See Intel 3a, Chapter 7, Section 7

/// In 64-bit mode the TSS holds information that is not
/// directly related to the task-switch mechanism,
/// but is used for finding kernel level stack
/// if interrupts arrive while in kernel mode.
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct TaskStateSegment {
    pub reserved: u32,
    /// The full 64-bit canonical forms of the stack pointers (RSP) for privilege levels 0-2.
    pub rsp: [u64; 3],
    pub reserved2: u64,
    /// The full 64-bit canonical forms of the interrupt stack table (IST) pointers.
    pub ist: [u64; 7],
    pub reserved3: u64,
    pub reserved4: u16,
    /// The 16-bit offset to the I/O permission bit map from the 64-bit TSS base.
    pub iomap_base: u16,
}

impl TaskStateSegment {
    pub const fn new() -> TaskStateSegment {
        TaskStateSegment {
            reserved: 0,
            rsp: [0; 3],
            reserved2: 0,
            ist: [0; 7],
            reserved3: 0,
            reserved4: 0,
            iomap_base: 0,
        }
    }
}
