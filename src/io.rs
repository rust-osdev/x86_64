/// Write 8 bits to port
pub unsafe fn outb(port: u16, val: u8) {
	asm!("outb %al, %dx" : : "{dx}"(port), "{al}"(val));
}

/// Read 8 bits from port
pub unsafe fn inb(port: u16) -> u8 {
	let ret : u8;
	asm!("inb %dx, %al" : "={ax}"(ret) : "{dx}"(port));
	return ret;
}

/// Write 16 bits to port
pub unsafe fn outw(port: u16, val: u16) {
	asm!("outw %ax, %dx" : : "{dx}"(port), "{al}"(val));
}

/// Read 16 bits from port
pub unsafe fn inw(port: u16) -> u16 {
	let ret : u16;
	asm!("inw %dx, %ax" : "={ax}"(ret) : "{dx}"(port));
	return ret;
}

/// Write 32 bits to port
pub unsafe fn outl(port: u16, val: u32) {
	asm!("outl %eax, %dx" : : "{dx}"(port), "{al}"(val));
}

/// Read 32 bits from port
pub unsafe fn inl(port: u16) -> u32 {
	let ret : u32;
	asm!("inl %dx, %eax" : "={ax}"(ret) : "{dx}"(port));
	return ret;
}
