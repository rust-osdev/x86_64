#[link(name = "x86_64_asm", kind = "static")]
extern "C" {
    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_interrupt_enable"
    )]
    pub(crate) fn x86_64_asm_interrupt_enable();

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_interrupt_disable"
    )]
    pub(crate) fn x86_64_asm_interrupt_disable();

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_interrupt_enable_and_hlt"
    )]
    pub(crate) fn x86_64_asm_interrupt_enable_and_hlt();

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_int3"
    )]
    pub(crate) fn x86_64_asm_int3();

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_hlt"
    )]
    pub(crate) fn x86_64_asm_hlt();

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_nop"
    )]
    pub(crate) fn x86_64_asm_nop();

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_read_from_port_u8"
    )]
    pub(crate) fn x86_64_asm_read_from_port_u8(port: u16) -> u8;

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_read_from_port_u16"
    )]
    pub(crate) fn x86_64_asm_read_from_port_u16(port: u16) -> u16;

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_read_from_port_u32"
    )]
    pub(crate) fn x86_64_asm_read_from_port_u32(port: u16) -> u32;

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_write_to_port_u8"
    )]
    pub(crate) fn x86_64_asm_write_to_port_u8(port: u16, value: u8);

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_write_to_port_u16"
    )]
    pub(crate) fn x86_64_asm_write_to_port_u16(port: u16, value: u16);

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_write_to_port_u32"
    )]
    pub(crate) fn x86_64_asm_write_to_port_u32(port: u16, value: u32);

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_set_cs"
    )]
    pub(crate) fn x86_64_asm_set_cs(sel: u64);

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_load_ss"
    )]
    pub(crate) fn x86_64_asm_load_ss(sel: u16);

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_load_ds"
    )]
    pub(crate) fn x86_64_asm_load_ds(sel: u16);

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_load_es"
    )]
    pub(crate) fn x86_64_asm_load_es(sel: u16);

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_load_fs"
    )]
    pub(crate) fn x86_64_asm_load_fs(sel: u16);

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_load_gs"
    )]
    pub(crate) fn x86_64_asm_load_gs(sel: u16);

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_swapgs"
    )]
    pub(crate) fn x86_64_asm_swapgs();

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_get_cs"
    )]
    pub(crate) fn x86_64_asm_get_cs() -> u16;

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_lgdt"
    )]
    pub(crate) fn x86_64_asm_lgdt(gdt: *const crate::instructions::tables::DescriptorTablePointer);

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_lidt"
    )]
    pub(crate) fn x86_64_asm_lidt(idt: *const crate::instructions::tables::DescriptorTablePointer);

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_ltr"
    )]
    pub(crate) fn x86_64_asm_ltr(sel: u16);

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_invlpg"
    )]
    pub(crate) fn x86_64_asm_invlpg(addr: u64);

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_read_cr0"
    )]
    pub(crate) fn x86_64_asm_read_cr0() -> u64;

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_write_cr0"
    )]
    pub(crate) fn x86_64_asm_write_cr0(value: u64);

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_read_cr2"
    )]
    pub(crate) fn x86_64_asm_read_cr2() -> u64;

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_read_cr3"
    )]
    pub(crate) fn x86_64_asm_read_cr3() -> u64;

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_write_cr3"
    )]
    pub(crate) fn x86_64_asm_write_cr3(value: u64);

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_read_cr4"
    )]
    pub(crate) fn x86_64_asm_read_cr4() -> u64;

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_write_cr4"
    )]
    pub(crate) fn x86_64_asm_write_cr4(value: u64);

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_rdmsr"
    )]
    pub(crate) fn x86_64_asm_rdmsr(msr: u32) -> u64;

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_wrmsr"
    )]
    pub(crate) fn x86_64_asm_wrmsr(msr: u32, value: u64);

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_read_rflags"
    )]
    pub(crate) fn x86_64_asm_read_rflags() -> u64;

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_write_rflags"
    )]
    pub(crate) fn x86_64_asm_write_rflags(val: u64);

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_rdfsbase"
    )]
    pub(crate) fn x86_64_asm_rdfsbase() -> u64;

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_wrfsbase"
    )]
    pub(crate) fn x86_64_asm_wrfsbase(val: u64);

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_rdgsbase"
    )]
    pub(crate) fn x86_64_asm_rdgsbase() -> u64;

    #[cfg_attr(
        any(target_env = "gnu", target_env = "musl"),
        link_name = "_x86_64_asm_wrgsbase"
    )]
    pub(crate) fn x86_64_asm_wrgsbase(val: u64);
}
