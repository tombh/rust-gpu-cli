//! Rust GPU shader compiler

#![feature(lint_reasons)]

mod builder;

use clap::Parser;

use builder::ShaderCLIArgs;

fn main() {
    tracing_subscriber::fmt::init();

    let args = ShaderCLIArgs::parse();
    args.start_shader_daemon();

    loop {
        std::thread::park();
    }
}
