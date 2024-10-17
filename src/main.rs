//! Rust GPU shader compiler

#![feature(lint_reasons)]

mod builder;
mod codegen_path;
mod rust_toolchain;
mod validate;

use clap::Parser;

use builder::ShaderCLIArgs;
use codegen_path::set_codegen_backend_path;
use rust_toolchain::ensure_rust_version;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    ensure_rust_version()?;
    set_codegen_backend_path()?;

    let args = ShaderCLIArgs::parse();
    args.start_shader_daemon();

    loop {
        std::thread::park();
    }
}
