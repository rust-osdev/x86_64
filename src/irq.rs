/// Enable Interrupts.
pub unsafe fn enable() 
{
    asm!("sti");
}

/// Disable Interrupts.
pub unsafe fn disable() 
{
    asm!("cli");
}

/// Generate a software interrupt. 
/// This is a macro argument needs to be an immediate.
#[macro_export]
macro_rules! int {
    ( $x:expr ) => {
        {
            asm!("int $0" :: "N" ($x));
        }
    };
}

/// A struct describing an interrupt gate.
#[derive(Debug, Copy)]
#[repr(packed)]
pub struct IdtEntry {
    /// Lower 16 bits of ISR.
    pub base_lo: u16,
    /// Segment selector.
    pub sel: u16,
    /// This must always be zero.
    pub res0: u8,
    /// Flags.
    pub flags: u8,
    /// The upper 48 bits of ISR (the last 16 bits must be zero).
    pub base_hi: u64,
    /// Must be zero.
    pub res1: u16
}

/// A struct describing a pointer to an array of interrupt handlers.
/// This is in a format suitable for giving to 'lidt'.
#[derive(Debug)]
#[repr(packed)]
pub struct IdtPointer
{
   /// Size of the IDT.
   pub limit: u16,
   /// Pointer to the memory region containing the IDT.
   pub base: u64
}

/// Load IDT table.
pub unsafe fn lidt(idt: u64)
{
    asm!("lidt (%rax)" :: "{rax}" (idt));
}


