#![feature(linkage, naked_functions, asm, const_fn)]
// Execute using: RUSTFLAGS="-C soft-float -C relocation-model=static -C code-model=kernel" RUST_BACKTRACE=1 cargo test --verbose --test kvm -- --nocapture

extern crate kvm;
extern crate memmap;
extern crate x86;
extern crate core;

#[macro_use]
extern crate klogger;

use kvm::{Capability, Exit, IoDirection, Segment, System, Vcpu, VirtualMachine};
use memmap::{Mmap, Protection};
use std::fs::File;
use std::io::{BufRead, BufReader};

use x86::shared::control_regs::*;
use x86::shared::paging::*;
use x86::bits64::paging::*;

unsafe fn use_the_port() {
    log!("1");
    asm!("inb $0, %al" :: "i"(0x01) :: "volatile");
}

#[test]
fn io_example() {
    let vaddr = VAddr::from_usize(use_the_port as *const () as usize);
    static PAGE_TABLE_P: PAddr = PAddr::from_u64(0x1000);

    // Set up a page table that identity maps the lower half of the address space
    let mut anon_mmap = Mmap::anonymous(3 * (1 << 20), Protection::ReadWrite).unwrap(); // Map 1 MiB
    let page_table_memory = unsafe { anon_mmap.as_mut_slice() };

    // "physical" layout of PageTable is: PML4Table at 0x1000, 1st PDPT at 0x2000, 2nd PDPT at 0x3000 ... 512th PDPT at 0x1000+512*0x1000
    type PageTable = (PML4, [PDPT; 512]);

    let page_table: &mut PageTable =
        unsafe { ::std::mem::transmute(&mut page_table_memory[PAGE_TABLE_P.as_u64() as usize]) };
    let (ref mut pml4, ref mut pdpts) = *page_table;
    // Identity map everything in PML4 slots 0..256
    for i in 0..512 {
        let offset = 0x2000 + 0x1000 * i;
        pml4[i] = PML4Entry::new(PAddr::from_u64(offset as _), PML4_P | PML4_RW);
        let pdpt = &mut pdpts[i];
        for j in 0..512 {
            pdpt[j] = PDPTEntry::new(PAddr::from_u64(((512 * i + j) as u64) << 30),
                                     PDPT_P | PDPT_RW | PDPT_PS); // Set-up 1 GiB page mappings
            if i == pml4_index(vaddr) && j == pdpt_index(vaddr) {
                println!("pml4_index(fn) {:x}", pml4[i].get_address());
                println!("pdpt_index(fn) {:x}", pdpt[j].get_address());
            }
        }
    }

    let mut stack_mmap = Mmap::anonymous(65536, Protection::ReadWrite).unwrap();
    let stack_size = stack_mmap.len();
    let stack_memory = unsafe { stack_mmap.as_mut_slice() };
    let stack_base = VAddr::from_usize(stack_memory.as_mut_ptr() as usize);
    static STACK_BASE_T: PAddr = PAddr::from_u64(0x2000000);
    println!("Stack base {:x} with size {:x}", stack_base, stack_size);

    // Initialize the KVM system
    let sys = System::initialize().unwrap();

    // Create a Virtual Machine
    let mut vm = VirtualMachine::create(&sys).unwrap();

    // Ensure that the VM supports memory backing with user memory
    assert!(vm.check_capability(Capability::UserMemory) > 0);

    // Once the memory is set we can't even call length.
    let page_table_memory_limit = page_table_memory.len() - 1;

    // Map the page table memory
    vm.set_user_memory_region(0, page_table_memory, 0).unwrap();
    // Map stack space
    vm.set_user_memory_region(STACK_BASE_T.as_u64(), stack_memory, 0).unwrap();

    // Map the process
    let f = File::open("/proc/self/maps").unwrap();
    let reader = BufReader::new(f);

    for line in reader.lines() {
        let line = line.unwrap();
        println!("{}", line);
        let mut s = line.split(' ');
        let mut s2 = s.next().unwrap().split('-');
        let begin = usize::from_str_radix(s2.next().unwrap(), 16).unwrap();
        let end = usize::from_str_radix(s2.next().unwrap(), 16).unwrap();
        if end < 0x800000000000 {
            let perm = s.next().unwrap();
            //println!("{:#X}-{:#X} {}", begin, end, perm);
            let slice = {
                let begin_ptr: *mut u8 = begin as *const u8 as _;
                unsafe { ::std::slice::from_raw_parts_mut(begin_ptr, end - begin) }
            };
            // Make sure process doesn't overlap with page table
            assert!(begin > page_table_memory_limit);
            vm.set_user_memory_region(begin as _, slice, 0).unwrap();
        }
    }

    // Create a new VCPU
    let mut vcpu = Vcpu::create(&mut vm).unwrap();

    // Set supported CPUID (KVM fails without doing this)
    let mut cpuid = sys.get_supported_cpuid().unwrap();
    /*for mut entry in cpuid.entries_mut() {
        if entry.function == 0x80000008 {
            entry.eax = entry.eax | 0xff;
        }
        println!("{:?}", entry);
    }*/
    vcpu.set_cpuid2(&mut cpuid).unwrap();

    // Setup the special registers
    let mut sregs = vcpu.get_sregs().unwrap();

    // Set the code segment to have base 0, limit 4GB (flat segmentation)
    let segment_template = Segment {
        base: 0x0,
        limit: 0xffffffff,
        selector: 0,
        _type: 0,
        present: 0,
        dpl: 0,
        db: 1,
        s: 0,
        l: 0,
        g: 1,
        avl: 0,
        ..Default::default()
    };

    sregs.cs = Segment {
        selector: 0x8,
        _type: 0xb,
        present: 1,
        db: 0,
        s: 1,
        l: 1,
        ..segment_template
    };
    sregs.ss = Segment { ..segment_template };
    sregs.ds = Segment { ..segment_template };
    sregs.es = Segment { ..segment_template };
    sregs.fs = Segment { ..segment_template };
    sregs.gs = Segment { ..segment_template };

    // We don't need to populate the GDT if we have our segments setup
    // cr0 - protected mode on, paging enabled
    sregs.cr0 = (CR0_PROTECTED_MODE | CR0_MONITOR_COPROCESSOR | CR0_EXTENSION_TYPE |
                 CR0_ENABLE_PAGING | CR0_NUMERIC_ERROR | CR0_WRITE_PROTECT |
                 CR0_ALIGNMENT_MASK | CR0_ENABLE_PAGING)
        .bits() as u64;
    sregs.cr3 = PAGE_TABLE_P.as_u64();
    sregs.cr4 = (CR4_ENABLE_PSE | CR4_ENABLE_PAE | CR4_ENABLE_GLOBAL_PAGES | CR4_ENABLE_SSE |
                 CR4_UNMASKED_SSE |
                 CR4_ENABLE_OS_XSAVE | CR4_ENABLE_SMEP | CR4_ENABLE_VME)
        .bits() as u64;
    sregs.efer = 0xd01;

    // Set the special registers
    vcpu.set_sregs(&sregs).unwrap();

    let mut regs = vcpu.get_regs().unwrap();
    // set the instruction pointer to 1 MB
    regs.rip = vaddr.as_usize() as u64;
    println!("regs.rip = 0x{:x}", regs.rip); // but is at: 0x40cd60
    //println!("regs.rip = 0x{:x}", unsafe { *(regs.rip as *const u64) }); // but is at: 0x40cd60
    //regs.rip = 0x40cd60;
    regs.rflags = 0x246;
    regs.rsp = STACK_BASE_T.as_u64() + stack_size as u64;
    regs.rbp = regs.rsp;
    vcpu.set_regs(&regs).unwrap();

    // Actually run the VCPU

    let mut ios_completes = 50;
    let mut new_regs = kvm::Regs::default();
    while ios_completes > 0 {
        {
            let (run, mut regs) = unsafe { vcpu.run_regs() }.unwrap();
            if run.exit_reason == Exit::Io {
                let io = unsafe { *run.io() };
                match io.direction {

                    IoDirection::In => {
                        if io.port == 0x3fd {
                            regs.rax = 0x20; // Mark serial line as ready to write
                        } else {
                            println!("IO on unknown port: {}", io.port);
                        }
                    }
                    IoDirection::Out => {
                        if io.port == 0x3f8 {
                            println!("got char {:#?}", regs.rax as u8 as char);
                        }
                        //println!("IOOut dont know what to do");
                    }
                }
                new_regs = regs;
            }
        }
        vcpu.set_regs(&new_regs).unwrap();

        ios_completes = ios_completes - 1;
    }

    // Ensure that the exit reason we get back indicates that the I/O
    // instruction was executed
    /*let regs = vcpu.get_regs().unwrap();
    println!("vcpu.rip: {:x}", regs.rip);

    assert!(run.exit_reason == Exit::Io);
    let io = unsafe { *run.io() };
    assert!(io.direction == IoDirection::In);
    assert!(io.size == 1);
    assert!(io.port == 0x1);
    unsafe {
        println!("{:#?}", *run.io());
    }*/
}
