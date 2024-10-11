# Rust GPU Shader Compiler

_Write GPU shaders in Rust_

Based on Embark Studios' `rust-gpu` project: https://github.com/Rust-GPU/rust-gpu. See their docs for details on how to actually write Rust shaders: https://rust-gpu.github.io/rust-gpu/book/

But here's the basic idea:

```rust
use spirv_std::glam::{Vec4, vec4};

#[spirv(fragment)]
pub fn main_fs(output: &mut Vec4) {
    *output = vec4(1.0, 0.0, 0.0, 1.0);
}
```

## Usage

```
Compile Rust shaders to SPIR-V. Runs as a daemon.

Usage: rust-gpu-compiler [OPTIONS] <PATH_TO_CRATE> [OUTPUT_PATH]

Arguments:
  <PATH_TO_CRATE>  Shader crate to compile
  [OUTPUT_PATH]    If set, combined SPIR-V and entrypoint metadata will be written to this file on succesful compile

Options:
  -t, --target <TARGET>
          rust-gpu compile target [default: spirv-unknown-spv1.3]
      --deny-warnings
          Treat warnings as errors during compilation
      --debug
          Compile shaders in debug mode
      --capability <CAPABILITY>
          Enables the provided SPIR-V capability
      --multimodule
          Compile one .spv file per entry point
      --spirv-metadata <SPIRV_METADATA>
          Set the level of metadata included in the SPIR-V binary [default: none]
      --relax-struct-store
          Allow store from one struct type to a different type with compatible layout and members
      --relax-logical-pointer
          Allow allocating an object of a pointer type and returning a pointer value from a function in logical addressing mode
      --relax-block-layout
          Enable VK_KHR_relaxed_block_layout when checking standard uniform, storage buffer, and push constant layouts. This is the default when targeting Vulkan 1.1 or later
      --uniform-buffer-standard-layout
          Enable VK_KHR_uniform_buffer_standard_layout when checking standard uniform buffer layouts
      --scalar-block-layout
          Enable VK_EXT_scalar_block_layout when checking standard uniform, storage buffer, and push constant layouts. Scalar layout rules are more permissive than relaxed block layout so in effect this will override the --relax-block-layout option
      --skip-block-layout
          Skip checking standard uniform / storage buffer layout. Overrides any --relax-block-layout or --scalar-block-layout option
      --preserve-bindings
          Preserve unused descriptor bindings. Useful for reflection
  -h, --help
          Print help
  -V, --version
          Print version
```

## Tips

- You can disassemble (inspect a text-readable version of) the resulting `.spv` files and even convert them to other formats like `.glsl` with Khronos' SPIR-V Tools: https://github.com/KhronosGroup/SPIRV-Tools. Pre-built binaries are available for most OSes.

## Rationale

- `rust-gpu` recommends including itself as a dependency of your main project. This generally works, but seeing as it is still in beta, it does have some rough edges and I've found that trying to debug them amongst my own projects' bugs, can make things overly complicated.
- `rust-gpu` pins to an entire Rust toolchain which can add unnecessary restrictions on your own project.
- I wonder if the compiler is a separate binary, then this is a better user experience anyway?

## TODO

- [ ] Will probably need to add multi-module support, see: https://github.com/EmbarkStudios/rust-gpu/issues/539
- [ ] Is it possible to make this into a single, publishable, standalone binary?
- [x] Always force build on first run? The `rust-gpu` watcher only rebuilds if the underlying shader code has changed. But changing the CLI args here should also be cause for re-compiling.

## Similar Projects

- The original CLI argument code for this project was taken from Bevy's https://github.com/Bevy-Rust-GPU/rust-gpu-builder
- This PR enables a dedicated CLI tool for building shaders:
  https://github.com/Bevy-Rust-GPU/rust-gpu-builder/pull/8

## Other `rust-gpu` Projects

- Issue discussing lack of docs: https://github.com/EmbarkStudios/rust-gpu/issues/1096
- https://github.com/EmbarkStudios/kajiya
- https://github.com/Patryk27/strolle
