#![feature(used, lang_items, const_fn)]

extern crate kvm;
extern crate memmap;
extern crate x86;

use kvm::{Capability, Exit, IoDirection, Segment, System, Vcpu, VirtualMachine};
use memmap::{Mmap, Protection};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::io;

use x86::shared::control_regs::*;
use x86::shared::paging::*;
use x86::bits64::paging::*;

#[no_mangle]
#[used]
pub static mut __TEST_PANICKED: bool = false;

struct PageTable {
    backing_memory: Mmap
}

type PageTableMemoryLayout = (PML4, [PDPT; 512]);

static PAGE_TABLE_P: PAddr = PAddr::from_u64(0x1000); // XXX:

impl PageTable {

    /// Allocated a chunk of memory to contain a basic page-table setup that covers the whole
    /// address space
    fn new() -> PageTable {
        let mut anon_mmap = Mmap::anonymous(3 * (1 << 20), Protection::ReadWrite).unwrap(); // Map 1 MiB
        PageTable { backing_memory: anon_mmap }
    }

    fn as_mut_slice<'a>(&'a mut self) -> &'a mut [u8] {
        unsafe { self.backing_memory.as_mut_slice() }
    }

    fn as_slice(&mut self) -> &[u8] {
        unsafe { self.backing_memory.as_slice() }
    }

    fn len(&mut self) -> usize {
        self.as_slice().len()
    }

    /// Constructs a simple page-table that identity maps
    /// the whole address space (guest virtual <-> guest physical).
    fn setup_identity_mapping(&mut self) {
        let page_table_memory = self.as_mut_slice();

        // "physical" layout of PageTable is:
        // PML4Table at 0x1000
        // 1st PDPT at 0x2000
        // 2nd PDPT at 0x3000 ...
        // 512th PDPT at 0x1000+512*0x1000
        // XXX: Can this be simpler why the offset?
        let page_table: &mut PageTableMemoryLayout =
            unsafe { ::std::mem::transmute(&mut page_table_memory[PAGE_TABLE_P.as_u64() as usize]) };

        // Identity map everything in our PML4:
        let (ref mut pml4, ref mut pdpts) = *page_table;
        for (i, mut pdpt) in pdpts.iter_mut().enumerate() {
            let offset = 0x2000 + 0x1000 * i;
            pml4[i] = PML4Entry::new(PAddr::from_u64(offset as _), PML4_P | PML4_RW);

            for j in 0..512 {
                // Set-up 1 GiB page-mappings:
                pdpt[j] = PDPTEntry::new(PAddr::from_u64(((512 * i + j) as u64) << 30),
                                         PDPT_P | PDPT_RW | PDPT_PS);
            }
        }
    }
}

struct Stack {
     backing_memory: Mmap
}

static STACK_BASE_T: PAddr = PAddr::from_u64(0x2000000);

impl Stack {

    /// Allocated a chunk of memory to contain a basic page-table setup that covers the whole
    /// address space
    fn new() -> Stack {
        let mut stack_mmap = Mmap::anonymous(65536, Protection::ReadWrite).unwrap();
        Stack { backing_memory: stack_mmap }
    }

    fn len(&self) -> usize {
        self.backing_memory.len()
    }

    fn as_mut_slice<'a>(&'a mut self) -> &'a mut [u8]{
        unsafe { self.backing_memory.as_mut_slice() }
    }

    fn size(&self) -> usize {
        self.backing_memory.len()
    }

    fn base(&self) -> VAddr {
        VAddr::from_usize(self.backing_memory.ptr() as usize)
    }
}

struct TestEnvironment<'a> {
    sys: &'a System,
    pt: &'a mut PageTable,
    st: &'a mut Stack,
    vm: VirtualMachine<'a>,
}

impl<'a> TestEnvironment<'a> {

    fn new(sys: &'a System, st: &'a mut Stack, pt: &'a mut PageTable) -> TestEnvironment<'a> {
        let mut vm = VirtualMachine::create(sys).unwrap();
        // Ensure that the VM supports memory backing with user memory
        assert!(vm.check_capability(Capability::UserMemory) > 0);

        /*let mut pt = PageTable::new();
        pt.setup_identity_mapping();
        let mut st = Stack::new();*/

        TestEnvironment { pt: pt, st: st, sys: sys, vm: vm }
    }

    /// Map the page table memory and stack memory
    fn create_vcpu(&'a mut self, init_fn: VAddr) -> kvm::Vcpu {
        // Once the memory is set we can't even call length.
        let page_table_memory_limit = self.pt.len() - 1;
        let stack_size = self.st.len();

        self.vm.set_user_memory_region(0, self.pt.as_mut_slice(), 0).unwrap();
        self.vm.set_user_memory_region(STACK_BASE_T.as_u64(), self.st.as_mut_slice(), 0).unwrap();

        // Map the process
        let f = File::open("/proc/self/maps").unwrap();
        let reader = BufReader::new(f);

        for line in reader.lines() {
            let line = line.unwrap();
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
                //assert!(begin > page_table_memory_limit);
                self.vm.set_user_memory_region(begin as _, slice, 0).unwrap();
            }
        }

        let mut vcpu = Vcpu::create(&mut self.vm).unwrap();
        // Set supported CPUID (KVM fails without doing this)
        let mut cpuid = self.sys.get_supported_cpuid().unwrap();
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
                     CR0_ALIGNMENT_MASK | CR0_ENABLE_PAGING).bits() as u64;
        sregs.cr3 = PAGE_TABLE_P.as_u64();
        sregs.cr4 = (CR4_ENABLE_PSE | CR4_ENABLE_PAE | CR4_ENABLE_GLOBAL_PAGES | CR4_ENABLE_SSE |
                     CR4_UNMASKED_SSE | CR4_ENABLE_OS_XSAVE | CR4_ENABLE_SMEP | CR4_ENABLE_VME)
                     .bits() as u64;
        sregs.efer = 0xd01; // XXX

        // Set the special registers
        vcpu.set_sregs(&sregs).unwrap();

        let mut regs = vcpu.get_regs().unwrap();

        // Set the instruction pointer to 1 MB
        regs.rip = init_fn.as_usize() as u64;
        regs.rflags = 0x246; // XXX
        regs.rsp = STACK_BASE_T.as_u64() + stack_size as u64;
        regs.rbp = regs.rsp;

        vcpu.set_regs(&regs).unwrap();

        vcpu
    }
}

#[derive(Debug)]
pub struct KvmTestMetaData {
    pub mbz: u64,
    pub meta: &'static str,
    pub identity_map: bool,
    pub physical_memory: (u64, u64),
    pub ioport_reads: (u16, u32),
}

/// Linker generates symbols that are inserted at the start and end of the kvm section.
extern "C" {
	static __start_kvm: std::os::raw::c_void;
	static __stop_kvm: std::os::raw::c_void;
}

/// Walks the kvm section to find see if there is a metha-data struct
/// lying around for the given test name.
fn find_meta_data(name: &str) -> Option<&KvmTestMetaData> {
    let (baseptr, size);

    // Safe: the linker will take care of initializing these symbols
	unsafe {
		baseptr = &__start_kvm as *const _ as *const KvmTestMetaData;
		size = &__stop_kvm as *const _ as usize - baseptr as usize;
	}

	let count = size / std::mem::size_of::<KvmTestMetaData>();

    // Safe: The section points to (static) KvmTestMetaData descriptions
	unsafe {
		let mods = std::slice::from_raw_parts(baseptr, count);
        for m in mods.iter() {
            if m.meta == name {
                return Some(&m);
            }
        }
	}

    None
}

/// Start the test harness.
pub fn test_start(ntests: usize) {
    println!("running {} tests (using KVM support)", ntests)
}

/// Signals that given test is ignored.
pub fn test_ignored(name: &str) {
    println!("test {} ... ignored", name);
}

pub fn test_before_run(name: &str) -> Option<&KvmTestMetaData> {
    println!("test {} ... ", name);
    find_meta_data(name)
}

pub fn test_panic_fmt(args: std::fmt::Arguments, file: &'static str, line: u32) {
    print!("\npanicked at '");
    use std::io::Write;
    std::io::stderr().write_fmt(args);
    println!("', {}:{}", file, line);
}

pub fn test_failed(_name: &str) {
    println!("FAILED");
}

pub fn test_success(_name: &str) {
    println!("OK");
}

pub fn test_summary(passed: usize, failed: usize, ignored: usize) {
    println!("\ntest result: {} {} passed; {} failed; {} ignored",
             if failed == 0 { "OK" } else { "FAILED" },
             passed,
             failed,
             ignored);

    if failed != 0 {
        std::process::exit(101);
    }
}

struct SerialPrinter {
    buffer: String
}

impl Write for SerialPrinter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        assert!(buf.len() == 1);
        self.buffer.push(buf[0] as char);
        match buf[0] as char {
            '\n' => {
                std::io::stdout().write(self.buffer.as_bytes());
                self.buffer.clear();
            }
            _ => {}
        }

        Ok(1)
    }

    fn flush(&mut self) -> io::Result<()> {
        std::io::stdout().write(self.buffer.as_bytes());
        self.buffer.clear();
        Ok(())
    }
}

impl SerialPrinter {
    fn new() -> SerialPrinter {
        SerialPrinter { buffer: String::new() }
    }
}

#[derive(Debug)]
enum IoHandleError {
    UnexpectedWrite(u16, u32),
    UnexpectedRead(u16)
}

enum IoHandleStatus {
    Handled,
    TestCompleted,
}

fn handle_ioexit(meta: &KvmTestMetaData, cpu: &mut Vcpu, run: &kvm::Run, printer: &mut SerialPrinter) -> Result<IoHandleStatus, IoHandleError> {
    let io = unsafe { *run.io() };

    match io.direction {
        IoDirection::In => {
            let mut regs = cpu.get_regs().unwrap();
            if io.port == 0x3fd {
                regs.rax = 0x20; // Mark serial line ready to write
                cpu.set_regs(&regs).unwrap();
                return Ok(IoHandleStatus::Handled);
            }
            else if io.port == meta.ioport_reads.0 {
                regs.rax = meta.ioport_reads.1 as u64;
                cpu.set_regs(&regs).unwrap();
                return Ok(IoHandleStatus::Handled);
            }
            return Err(IoHandleError::UnexpectedRead(io.port));
        }
        IoDirection::Out => {
            let regs = cpu.get_regs().unwrap();
            if io.port == 0x3f8 {
                printer.write(&[regs.rax as u8]);
                return Ok(IoHandleStatus::Handled);
            }
            else if io.port == 0xf4 && regs.rax as u8 == 0x0 {
                // Magic shutdown command for exiting the test.
                // The line unsafe { x86::shared::io::outw(0xf4, 0x00); }
                // is automatically inserted at the end of every test!
                return Ok(IoHandleStatus::TestCompleted);
            }

            return Err(IoHandleError::UnexpectedWrite(io.port, regs.rax as u32));
        }
    };
}

pub fn test_main_static(tests: &[TestDescAndFn]) {
    test_start(tests.len());

    let mut failed = 0;
    let mut ignored = 0;
    let mut passed = 0;
    for test in tests {
        if test.desc.ignore {
            ignored += 1;
            test_ignored(test.desc.name.0);
        } else {
            let meta_data = test_before_run(test.desc.name.0);

            unsafe { __TEST_PANICKED = false; }

            match meta_data {
                // Run this in a Virtual machine
                Some(mtd) => {
                    let sys = System::initialize().unwrap();
                    let mut st = Stack::new();
                    let mut pt = PageTable::new();
                    pt.setup_identity_mapping();
                    let mut test_environment = TestEnvironment::new(&sys, &mut st, &mut pt);
                    let mut printer: SerialPrinter = SerialPrinter::new();

                    let test_fn_vaddr = VAddr::from_usize(test.testfn.0 as *const () as usize);
                    let mut vcpu = test_environment.create_vcpu(test_fn_vaddr);

                    let mut vm_is_done = false;
                    while !vm_is_done {
                        let run = unsafe { vcpu.run() }.unwrap();
                        match run.exit_reason {
                            Exit::Io => {
                                match handle_ioexit(&mtd, &mut vcpu, &run, &mut printer) {
                                    Result::Ok(IoHandleStatus::Handled) => {/* Continue */}
                                    Result::Ok(IoHandleStatus::TestCompleted) => vm_is_done = true,
                                    Result::Err(err) => {
                                        println!("Test failed due to unexpected IO: {:?}", err);
                                        vm_is_done = true;
                                    }
                                }
                            },
                            Exit::Shutdown => {
                                println!("Exit::Shutdown");
                                vm_is_done = true;
                            }
                            _ => {
                                println!("Unknown exit reason: {:?}", run.exit_reason);
                            }
                        }
                    }

                },
                // Regular test, execute as usual:
                _ => test.testfn.0() // Regular test, not running inside virtual machine
            }

            unsafe {
                if __TEST_PANICKED == (test.desc.should_panic == ShouldPanic::Yes) {
                    passed += 1;
                    test_success(test.desc.name.0);
                } else {
                    failed += 1;
                    test_failed(test.desc.name.0);
                }
            }
        }

    }

    test_summary(passed, failed, ignored);
}

// required for compatibility with the `rustc --test` interface
pub struct TestDescAndFn {
    pub desc: TestDesc,
    pub testfn: StaticTestFn,
}

pub struct TestDesc {
    pub ignore: bool,
    pub name: StaticTestName,
    pub should_panic: ShouldPanic,
}

pub struct StaticTestName(pub &'static str);
pub struct StaticTestFn(pub fn());

#[derive(PartialEq)]
pub enum ShouldPanic {
    No,
    Yes,
}
