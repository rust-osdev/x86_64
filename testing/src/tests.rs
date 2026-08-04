#[test_case]
fn example_test() {
    serial_print!("example_test... ");
    assert_eq!(0, 0);
    serial_println!("[ok]");
}

#[test_case]
fn runtime_virtual_address_validity_in_la48() {
    use x86_64::registers::control::{Cr4, Cr4Flags};
    use x86_64::{VirtAddr48, VirtAddr57, VirtAddrRT};

    serial_print!("runtime_virtual_address_validity_in_la48... ");
    assert!(!Cr4::read().contains(Cr4Flags::L5_PAGING));
    assert!(VirtAddrRT::try_new(0x0000_7fff_ffff_ffff).is_ok());
    assert!(VirtAddrRT::try_new(0x00ff_ffff_ffff_ffff).is_err());
    assert!(VirtAddr48::new_const(0x1234).is_valid_currently());
    assert!(!VirtAddr57::new_const(0x00ff_0000_0000_0000).is_valid_currently());
    serial_println!("[ok]");
}
