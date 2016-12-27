//! I/O port functionality.

/// Write 8 bits to port
pub unsafe fn outb(port: u16, val: u8) {
    asm!("outb %al, %dx" :: "{dx}"(port), "{al}"(val));
}

/// Read 8 bits from port
pub unsafe fn inb(port: u16) -> u8 {
    let ret: u8;
    asm!("inb %dx, %al" : "={ax}"(ret) : "{dx}"(port) :: "volatile");
    ret
}

/// Write 16 bits to port
pub unsafe fn outw(port: u16, val: u16) {
    asm!("outw %ax, %dx" :: "{dx}"(port), "{al}"(val));
}

/// Read 16 bits from port
pub unsafe fn inw(port: u16) -> u16 {
    let ret: u16;
    asm!("inw %dx, %ax" : "={ax}"(ret) : "{dx}"(port) :: "volatile");
    ret
}

/// Write 32 bits to port
pub unsafe fn outl(port: u16, val: u32) {
    asm!("outl %eax, %dx" :: "{dx}"(port), "{al}"(val));
}

/// Read 32 bits from port
pub unsafe fn inl(port: u16) -> u32 {
    let ret: u32;
    asm!("inl %dx, %eax" : "={ax}"(ret) : "{dx}"(port) :: "volatile");
    ret
}


// Write 8-bit array to port
pub unsafe fn outsb(port: u16, buf: &[u8]) {
    asm!("rep outsb (%esi), %dx"
         :: "{ecx}"(buf.len()), "{dx}"(port), "{esi}"(buf.as_ptr())
         : "ecx", "edi");
}

// Read 8-bit array from port
pub unsafe fn insb(port: u16, buf: &mut [u8]) {
    asm!("rep insb %dx, (%edi)"
         :: "{ecx}"(buf.len()), "{dx}"(port), "{edi}"(buf.as_ptr())
         : "ecx", "edi" : "volatile");
}

// Write 16-bit array to port
pub unsafe fn outsw(port: u16, buf: &[u16]) {
    asm!("rep outsw (%esi), %dx"
         :: "{ecx}"(buf.len()), "{dx}"(port), "{esi}"(buf.as_ptr())
         : "ecx", "edi");
}

// Read 16-bit array from port
pub unsafe fn insw(port: u16, buf: &mut [u16]) {
    asm!("rep insw %dx, (%edi)"
         :: "{ecx}"(buf.len()), "{dx}"(port), "{edi}"(buf.as_ptr())
         : "ecx", "edi" : "volatile");
}

// Write 32-bit array to port
pub unsafe fn outsl(port: u16, buf: &[u32]) {
    asm!("rep outsl (%esi), %dx"
         :: "{ecx}"(buf.len()), "{dx}"(port), "{esi}"(buf.as_ptr())
         : "ecx", "edi");
}

// Read 32-bit array from port
pub unsafe fn insl(port: u16, buf: &mut [u32]) {
    asm!("rep insl %dx, (%edi)"
         :: "{ecx}"(buf.len()), "{dx}"(port), "{edi}"(buf.as_ptr())
         : "ecx", "edi" : "volatile");
}
