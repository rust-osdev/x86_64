#![feature(used, lang_items)]

pub fn test_start(ntests: usize) {
    println!("KVM testing: running {} tests", ntests)
}

pub fn test_ignored(name: &str) {
    println!("test {} ... ignored", name);
}

pub fn test_before_run(name: &str) {
    print!("test {} ... ", name);
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

#[no_mangle]
#[used]
pub static mut __TEST_PANICKED: bool = false;

pub fn test_main_static(tests: &[TestDescAndFn]) {
    unsafe {
        test_start(tests.len());

        let mut failed = 0;
        let mut ignored = 0;
        let mut passed = 0;
        for test in tests {
            if test.desc.ignore {
                ignored += 1;
                test_ignored(test.desc.name.0);
            } else {
                test_before_run(test.desc.name.0);

                __TEST_PANICKED = false;

                test.testfn.0();

                if __TEST_PANICKED == (test.desc.should_panic == ShouldPanic::Yes) {
                    passed += 1;
                    test_success(test.desc.name.0);
                } else {
                    failed += 1;
                    test_failed(test.desc.name.0);
                }
            }

        }

        test_summary(passed, failed, ignored);
    }
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
