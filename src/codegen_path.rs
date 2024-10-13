//! `rust-gpu` internally calls `rustc` with the `-Zcodegen-backend=<path>` argument so that the
//! Rust compiler knows how to target SPIR-V. The codegen backend is `librustc_codegen_spirv.so`
//! and is the result of compiling `spirv-builder`. `rust-gpu` currently derives the path to
//! `librustc_codegen_spirv.so` from whatever ENV var the active OS uses to specify dynamic
//! library paths (.so, .dll etc). This is because `rust-gpu` doesn't yet have official
//! releases and so can depends `cargo` setting the dynamic library path var, which points to
//! `target/release/`.
//!
//! In an actual formal install we want to put `librustc_codegen_spirv.so` in one of the system
//! paths that contains other dynamic libraries (`/usr/lib` for exmaple). Note that we can't rely
//! on conventional lookup methods, like `ldconfig` for example, because `rust-gpu` needs the
//! literal string `librustc_codegen_spirv.so` in a CLI argument.
//!
//! I'm sure once `rust-gpu` gets nearer an official release, this will all be improved. Maybe
//! `librustc_codegen_spirv.so` can be statically linked?

/// Inject the path to `librustc_codegen_spirv.so` into the OS's dynamic library ENV.
pub fn set_codegen_backend_path() -> anyhow::Result<()> {
    let dylib_var = dylib_path_envvar();

    let mut paths = Vec::new();
    if let Some(path) = std::env::var_os(dylib_var) {
        paths = std::env::split_paths(&path).collect::<Vec<_>>();
    }
    paths.push(std::path::PathBuf::from(codegen_spirv_path()));
    let new_path = std::env::join_paths(paths)?;
    std::env::set_var(dylib_var, new_path);

    Ok(())
}

/// Get the ENV variable name for the list of paths pointing to .so/.dll files
const fn dylib_path_envvar() -> &'static str {
    if cfg!(windows) {
        "PATH"
    } else if cfg!(target_os = "macos") {
        "DYLD_FALLBACK_LIBRARY_PATH"
    } else {
        "LD_LIBRARY_PATH"
    }
}

/// Get the ENV variable name for the list of paths pointing to .so/.dll files
const fn codegen_spirv_path() -> &'static str {
    if cfg!(windows) {
        "C:\\Windows\\System32"
    } else if cfg!(target_os = "macos") {
        "/Applications/rust-gpu-compiler"
    } else {
        "/usr/lib"
    }
}
