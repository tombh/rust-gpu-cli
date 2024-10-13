//! `rust-gpu` depends on Rust Nightly features. Here we enforce the required version of Rust and
//! its toolchain. By ensuring the version here that means that the shader code itself doesn't need
//! to depend on that version. Or in other words, compiling the shader for GPU depends on `rust-gpu`'s
//! Rust version, but compiling the shader for CPU doesn't depend on `rust-gpu`'s requirements.
//!

use anyhow::Context;

/// Set the `RUSTUP_TOOLCHAIN` ENV var based on the value set in `rust-toolchain.toml`.
pub fn ensure_rust_version() -> anyhow::Result<()> {
    let bytes = include_bytes!("../rust-toolchain.toml");
    let string = String::from_utf8_lossy(bytes);
    let whole_toml_file = string.parse::<toml::Table>()?;
    let toolchain = whole_toml_file
        .get("toolchain")
        .context("Internal: `toolchain` not set in embedded `rust-toolchain.toml`")?;
    let binding = toolchain
        .get("channel")
        .context("Internal: `channel` not set in embedded `rust-toolchain.toml`")?
        .to_string();
    let channel = binding.trim_matches('"');
    std::env::set_var("RUSTUP_TOOLCHAIN", channel);
    tracing::debug!("`RUSTUP_TOOLCHAIN` ENV set to: {channel}");
    Ok(())
}
