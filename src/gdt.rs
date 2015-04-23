/// In 64-bit mode the TSS holds information that is not
/// directly related to the task-switch mechanism.
#[repr(C, packed)]
pub struct TaskStateSegement {
    reserved: u32,
    /// The full 64-bit canonical forms of the stack pointers (RSP) for privilege levels 0-2.
    rsp: [u64; 3],
    reserved2: u64,
    /// The full 64-bit canonical forms of the interrupt stack table (IST) pointers.
    ist: [u64; 7],
    reserved3 [u64],
    reserved4: u16,
    /// The 16-bit offset to the I/O permission bit map from the 64-bit TSS base.
    iomap_base: u16,
}

#[repr(C, packed)]
pub struct RegionDescriptor {
    /// Segement extent
    limit: u16,
    /// Base address
    base: u64,
}


pub struct GdtEntry {

}