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

/// Load IDT table.
pub unsafe fn lidt(idt: u64)
{
    asm!("lidt (%rax)" :: "{rax}" (idt));
}


