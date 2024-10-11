//! Rust GPU shader compiler

// Lints are also specified in `Cargo.toml`, but the current Rust toolchain doesn't support those.
// These can be removed once the toolchain is updated.
#![warn(
    clippy::all,
    missing_docs,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
#![allow(clippy::blanket_clippy_restriction_lints)]
#![warn(clippy::restriction)]
#![allow(clippy::implicit_return)]

mod builder;

use std::time::Duration;

use clap::Parser;
use tracing_subscriber;

use builder::ShaderBuilder;

fn main() {
    tracing_subscriber::fmt::init();

    let args = ShaderBuilder::parse();
    args.start_shader_daemon();

    loop {
        std::thread::park();
    }
}
