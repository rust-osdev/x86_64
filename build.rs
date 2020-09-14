fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(all(feature = "external_asm", windows))]
    compile_error!("\"external_asm\" feature is not available on windows toolchain!");

    #[cfg(feature = "instructions")]
    if std::env::var("CARGO_CFG_TARGET_ARCH").unwrap() != "x86_64" {
        panic!("\"instructions\" feature is only available for x86_64 targets!");
    }

    #[cfg(all(
        feature = "instructions",
        not(feature = "inline_asm"),
        not(feature = "external_asm")
    ))]
    compile_error!("\"instructions\" feature is enabled, but neither feature \"external_asm\" nor \"inline_asm\" was set!");

    #[cfg(all(feature = "inline_asm", feature = "external_asm"))]
    compile_error!(
        "\"inline_asm\" and \"external_asm\" features can not be enabled at the same time!"
    );

    #[cfg(all(feature = "instructions", feature = "external_asm"))]
    {
        use std::ffi::OsString;
        use std::fs;

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
}
