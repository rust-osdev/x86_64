.text
.code64

# REMEMBER: This code uses the AMD64 calling convention:
#   Arguments: RDI, RSI, RDX, RCX
#   Return: RAX

.global _x86_64_asm_interrupt_enable
.p2align 4
_x86_64_asm_interrupt_enable:
    sti
    retq

.global _x86_64_asm_interrupt_disable
.p2align 4
_x86_64_asm_interrupt_disable:
    cli
    retq

.global _x86_64_asm_interrupt_enable_and_hlt
.p2align 4
_x86_64_asm_interrupt_enable_and_hlt:
    sti
    hlt
    retq

.global _x86_64_asm_int3
.p2align 4
_x86_64_asm_int3:
    int3
    retq

.global _x86_64_asm_read_from_port_u8
.p2align 4
_x86_64_asm_read_from_port_u8:
    mov    %edi, %edx
    inb    (%dx), %al
    retq

.global _x86_64_asm_read_from_port_u16
.p2align 4
_x86_64_asm_read_from_port_u16:
    mov    %edi, %edx
    inw    (%dx), %ax
    retq

.global _x86_64_asm_read_from_port_u32
.p2align 4
_x86_64_asm_read_from_port_u32:
    mov    %edi, %edx
    inl    (%dx), %eax
    retq


.global _x86_64_asm_write_to_port_u8
.p2align 4
_x86_64_asm_write_to_port_u8:
    mov    %edi, %edx
    mov    %si, %ax
    outb   %al, (%dx)
    retq

.global _x86_64_asm_write_to_port_u16
.p2align 4
_x86_64_asm_write_to_port_u16:
    mov    %edi, %edx
    mov    %si, %ax
    outw   %ax, (%dx)
    retq

.global _x86_64_asm_write_to_port_u32
.p2align 4
_x86_64_asm_write_to_port_u32:
    mov    %edi, %edx
    mov    %esi, %eax
    outl   %eax, (%dx)
    retq

.global _x86_64_asm_set_cs
.p2align 4
_x86_64_asm_set_cs:
    pushq %rdi
    leaq  1f(%rip), %rax
    pushq %rax
    lretq
1:
    retq

.global _x86_64_asm_get_cs
.p2align 4
_x86_64_asm_get_cs:
    mov %cs, %ax
    retq

.global _x86_64_asm_invlpg
.p2align 4
_x86_64_asm_invlpg:
    invlpg (%rdi)
    retq

.global _x86_64_asm_invpcid
.p2align 4
_x86_64_asm_invpcid:
    invpcid (%rsi), %rdi
    retq

.global _x86_64_asm_ltr
.p2align 4
_x86_64_asm_ltr:
    mov %edi, %edx
    ltr %dx
    retq

.global _x86_64_asm_lgdt
.p2align 4
_x86_64_asm_lgdt:
    lgdt (%rdi)
    retq

.global _x86_64_asm_lidt
.p2align 4
_x86_64_asm_lidt:
    lidt (%rdi)
    retq

.global _x86_64_asm_sgdt
.p2align 4
_x86_64_asm_sgdt:
    sgdt (%rdi)
    retq

.global _x86_64_asm_sidt
.p2align 4
_x86_64_asm_sidt:
    sidt (%rdi)
    retq

.global _x86_64_asm_write_rflags
.p2align 4
_x86_64_asm_write_rflags:
    pushq %rdi
    popfq
    retq

.global _x86_64_asm_read_rflags
.p2align 4
_x86_64_asm_read_rflags:
    pushfq
    popq    %rax
    retq

.global _x86_64_asm_load_ss
.p2align 4
_x86_64_asm_load_ss:
    mov %di, %ss
    retq

.global _x86_64_asm_load_ds
.p2align 4
_x86_64_asm_load_ds:
    mov %di, %ds
    retq

.global _x86_64_asm_load_es
.p2align 4
_x86_64_asm_load_es:
    mov %di, %es
    retq

.global _x86_64_asm_load_fs
.p2align 4
_x86_64_asm_load_fs:
    mov %di, %fs
    retq

.global _x86_64_asm_load_gs
.p2align 4
_x86_64_asm_load_gs:
    mov %di, %gs
    retq

.global _x86_64_asm_get_ss
.p2align 4
_x86_64_asm_get_ss:
    mov %ss, %ax
    retq

.global _x86_64_asm_get_ds
.p2align 4
_x86_64_asm_get_ds:
    mov %ds, %ax
    retq

.global _x86_64_asm_get_es
.p2align 4
_x86_64_asm_get_es:
    mov %es, %ax
    retq

.global _x86_64_asm_get_fs
.p2align 4
_x86_64_asm_get_fs:
    mov %fs, %ax
    retq

.global _x86_64_asm_get_gs
.p2align 4
_x86_64_asm_get_gs:
    mov %gs, %ax
    retq

.global _x86_64_asm_swapgs
.p2align 4
_x86_64_asm_swapgs:
    swapgs
    retq

.global _x86_64_asm_read_cr0
.p2align 4
_x86_64_asm_read_cr0:
    movq %cr0, %rax
    retq

.global _x86_64_asm_read_cr2
.p2align 4
_x86_64_asm_read_cr2:
    movq %cr2, %rax
    retq

.global _x86_64_asm_read_cr3
.p2align 4
_x86_64_asm_read_cr3:
    movq %cr3, %rax
    retq

.global _x86_64_asm_read_cr4
.p2align 4
_x86_64_asm_read_cr4:
    movq %cr4, %rax
    retq

.global _x86_64_asm_write_cr0
.p2align 4
_x86_64_asm_write_cr0:
    movq %rdi, %cr0
    retq

.global _x86_64_asm_write_cr3
.p2align 4
_x86_64_asm_write_cr3:
    movq %rdi, %cr3
    retq

.global _x86_64_asm_write_cr4
.p2align 4
_x86_64_asm_write_cr4:
    movq %rdi, %cr4
    retq

.global _x86_64_asm_rdmsr
.p2align 4
_x86_64_asm_rdmsr:
    mov    %edi, %ecx    # First param is the MSR number
    rdmsr
    shl    $32,  %rdx    # shift edx to upper 32bit
    mov    %eax, %eax    # clear upper 32bit of rax
    or     %rdx, %rax    # or with rdx
    retq

.global _x86_64_asm_wrmsr
.p2align 4
_x86_64_asm_wrmsr:
    movl  %edi, %ecx    # First param is the MSR number
    movl  %esi, %eax    # Second param is the low 32-bits
    wrmsr               # Third param (high 32-bits) is already in %edx
    retq

.global _x86_64_asm_hlt
.p2align 4
_x86_64_asm_hlt:
    hlt
    retq

.global _x86_64_asm_nop
.p2align 4
_x86_64_asm_nop:
    nop
    retq

.global _x86_64_asm_bochs
.p2align 4
_x86_64_asm_bochs:
    xchgw %bx, %bx
    retq

.global _x86_64_asm_rdfsbase
.p2align 4
_x86_64_asm_rdfsbase:
    rdfsbase %rax
    retq

.global _x86_64_asm_wrfsbase
.p2align 4
_x86_64_asm_wrfsbase:
    wrfsbase %rdi
    retq

.global _x86_64_asm_rdgsbase
.p2align 4
_x86_64_asm_rdgsbase:
    rdgsbase %rax
    retq

.global _x86_64_asm_wrgsbase
.p2align 4
_x86_64_asm_wrgsbase:
    wrgsbase %rdi
    retq

.global _x86_64_asm_xgetbv
.p2align 4
_x86_64_asm_xgetbv:
    mov    %edi, %ecx    # First param is the XCR number
    xgetbv
    shl    $32,  %rdx    # shift edx to upper 32bit
    mov    %eax, %eax    # clear upper 32bit of rax
    or     %rdx, %rax    # or with rdx
    retq

.global _x86_64_asm_xsetbv
.p2align 4
_x86_64_asm_xsetbv:
    movl  %edi, %ecx    # First param is the XCR number
    movl  %esi, %eax    # Second param is the low 32-bits
    xsetbv              # Third param (high 32-bits) is already in %edx
    retq
