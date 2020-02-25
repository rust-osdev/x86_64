#[cfg(feature = "inline_asm")]
fn main() {}

#[cfg(all(not(feature = "inline_asm"), not(feature = "stable")))]
fn main() {
    compile_error!("Neither feature \"stable\" nor \"inline_asm\" was set!");
}

#[cfg(all(not(feature = "inline_asm"), feature = "stable"))]
fn main() {
    use std::ffi::OsString;
    use std::fs;

    println!("cargo:rerun-if-changed=build.rs");

    let entries = fs::read_dir("src/asm")
        .unwrap()
        .filter_map(|f| {
            f.ok().and_then(|e| {
                let path = e.path();
                match path.extension() {
                    Some(ext) if ext.eq(&OsString::from("s")) => Some(path),
                    _ => None,
                }
            })
        })
        .collect::<Vec<_>>();

    cc::Build::new()
        .no_default_flags(true)
        .files(&entries)
        .pic(true)
        .static_flag(true)
        .shared_flag(false)
        .compile("x86_64_asm");

    for e in entries {
        println!("cargo:rerun-if-changed={}", e.to_str().unwrap());
    }
}
