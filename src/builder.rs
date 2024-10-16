//! Builder daemon to compile Rust shaders into SPIR-V.

use core::borrow::Borrow;
use core::str::FromStr;
use std::path::PathBuf;

use spirv_builder::CompileResult;

use crate::validate::validate;

/// CLI arguments
#[expect(
    clippy::struct_excessive_bools,
    reason = "We're just a simple CLI app, we don't need a state machine"
)]
#[derive(Debug, Clone, clap::Parser)]
#[command(author, version, about, long_about = None)]
pub struct ShaderCLIArgs {
    /// Shader crate to compile.
    path_to_crate: PathBuf,

    /// If set, shader module will be copied here. Otherwise shader module is copied to the root of
    /// the shader crate at `compiled/[crate name].spv`, see logs for exact path.
    output_path: Option<PathBuf>,

    /// rust-gpu compile target.
    #[arg(short, long, default_value = "spirv-unknown-spv1.3")]
    target: String,

    /// Treat warnings as errors during compilation.
    #[arg(long, default_value = "false")]
    deny_warnings: bool,

    /// Compile shaders in debug mode.
    #[arg(long, default_value = "false")]
    debug: bool,

    /// Enables the provided SPIR-V capabilities.
    /// See: `impl core::str::FromStr for spirv_builder::Capability`
    #[arg(long, value_parser=Self::spirv_capability)]
    capability: Vec<spirv_builder::Capability>,

    /// Enables the provided SPIR-V extensions.
    /// See https://github.com/KhronosGroup/SPIRV-Registry for all extensions
    #[arg(long)]
    extension: Vec<String>,

    /// Compile one .spv file per entry point.
    #[arg(long, default_value = "false")]
    multimodule: bool,

    /// Set the level of metadata included in the SPIR-V binary.
    #[arg(long, value_parser=Self::spirv_metadata, default_value = "none")]
    spirv_metadata: spirv_builder::SpirvMetadata,

    /// Allow store from one struct type to a different type with compatible layout and members.
    #[arg(long, default_value = "false")]
    relax_struct_store: bool,

    /// Allow allocating an object of a pointer type and returning a pointer value from a function
    /// in logical addressing mode.
    #[arg(long, default_value = "false")]
    relax_logical_pointer: bool,

    /// Enable VK_KHR_relaxed_block_layout when checking standard uniform,
    /// storage buffer, and push constant layouts.
    /// This is the default when targeting Vulkan 1.1 or later.
    #[arg(long, default_value = "false")]
    relax_block_layout: bool,

    /// Enable VK_KHR_uniform_buffer_standard_layout when checking standard uniform buffer layouts.
    #[arg(long, default_value = "false")]
    uniform_buffer_standard_layout: bool,

    /// Enable VK_EXT_scalar_block_layout when checking standard uniform, storage buffer, and push
    /// constant layouts.
    /// Scalar layout rules are more permissive than relaxed block layout so in effect this will
    /// override the --relax-block-layout option.
    #[arg(long, default_value = "false")]
    scalar_block_layout: bool,

    /// Skip checking standard uniform / storage buffer layout. Overrides any --relax-block-layout
    /// or --scalar-block-layout option.
    #[arg(long, default_value = "false")]
    skip_block_layout: bool,

    /// Preserve unused descriptor bindings. Useful for reflection.
    #[arg(long, default_value = "false")]
    preserve_bindings: bool,

    /// Validate the compiled SPIR-V binary and, optionally, its WGSL version using `naga`
    /// Options:
    ///   - "spirv": validates the generated SPIR-V binary
    ///   - "wgsl": cross-compiles the SPIR-V binary to WGSL, and also validates the WGSL
    #[arg(long, value_parser=Self::validation, verbatim_doc_comment)]
    validate: Option<ValidationOption>,
}

/// Options for SPIR-V validation.
#[derive(Clone, Copy, Debug)]
enum ValidationOption {
    /// Only validate the generated SPIR-V module.
    Spriv,
    /// Also create a WGSL version of the SPIR-V module and validate that WGSL.
    Wgsl,
}

impl ShaderCLIArgs {
    /// Clap value parser for `SpirvMetadata`.
    fn spirv_metadata(metadata: &str) -> Result<spirv_builder::SpirvMetadata, clap::Error> {
        match metadata {
            "none" => Ok(spirv_builder::SpirvMetadata::None),
            "name-variables" => Ok(spirv_builder::SpirvMetadata::NameVariables),
            "full" => Ok(spirv_builder::SpirvMetadata::Full),
            _ => Err(clap::Error::new(clap::error::ErrorKind::InvalidValue)),
        }
    }

    /// Clap value parser for validation options.
    fn validation(validation: &str) -> Result<ValidationOption, clap::Error> {
        match validation {
            "spirv" => Ok(ValidationOption::Spriv),
            "wgsl" => Ok(ValidationOption::Wgsl),
            _ => Err(clap::Error::new(clap::error::ErrorKind::InvalidValue)),
        }
    }

    /// Clap value parser for `Capability`.
    fn spirv_capability(capability: &str) -> Result<spirv_builder::Capability, clap::Error> {
        spirv_builder::Capability::from_str(capability).map_or_else(
            |()| Err(clap::Error::new(clap::error::ErrorKind::InvalidValue)),
            Ok,
        )
    }

    /// Create the SPIR-V builder from the given CLI args.
    fn make_builder(&self) -> spirv_builder::SpirvBuilder {
        let mut builder = spirv_builder::SpirvBuilder::new(&self.path_to_crate, &self.target)
            .deny_warnings(self.deny_warnings)
            .release(!self.debug)
            .multimodule(self.multimodule)
            .spirv_metadata(self.spirv_metadata)
            .relax_struct_store(self.relax_struct_store)
            .relax_logical_pointer(self.relax_logical_pointer)
            .relax_block_layout(self.relax_block_layout)
            .uniform_buffer_standard_layout(self.uniform_buffer_standard_layout)
            .scalar_block_layout(self.scalar_block_layout)
            .skip_block_layout(self.skip_block_layout)
            .preserve_bindings(self.preserve_bindings)
            .print_metadata(spirv_builder::MetadataPrintout::None);

        for capability in &self.capability {
            builder = builder.capability(*capability);
        }

        for extension in &self.extension {
            builder = builder.extension(extension);
        }

        builder
    }

    /// Starts watching a shader directory and compiles on changes
    #[expect(clippy::expect_used, reason = "We can panic at startup")]
    pub fn start_shader_daemon(&self) {
        tracing::info!("Starting daemon");

        let builder = self.make_builder();

        let source = self.path_to_crate.clone();
        let is_custom_output_path = self.output_path.is_some();
        let destination_path = self
            .output_path
            .borrow()
            .as_ref()
            .map_or_else(|| source.join("compiled"), core::clone::Clone::clone);
        let destination_string = destination_path
            .into_os_string()
            .into_string()
            .expect("Couldn't parse destination path");
        let destination_for_watcher = destination_string.clone();

        let validation = self.validate;

        let first_compile_result = builder
            .watch(move |compile_result| {
                let destination_clone = destination_for_watcher.clone();
                Self::handle_compile_result(
                    &compile_result,
                    is_custom_output_path,
                    destination_clone,
                    validation,
                );
            })
            .expect("First compile failed");

        Self::handle_compile_result(
            &first_compile_result,
            is_custom_output_path,
            destination_string,
            validation,
        );
    }

    /// Handle the result of a Rust-to-SPIRV compilation.
    fn handle_compile_result(
        compile_result: &CompileResult,
        is_custom_output_path: bool,
        destination: String,
        maybe_validation: Option<ValidationOption>,
    ) {
        let destination_path: PathBuf = destination.into();
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "`single` is a value but `&compile_result.module` is a ref?"
        )]
        match &compile_result.module {
            spirv_builder::ModuleResult::SingleModule(single) => {
                let mut copy_to = destination_path.clone();

                #[expect(
                    clippy::expect_used,
                    reason = "There's no way to continue if we can't create the destination directory"
                )]
                if !is_custom_output_path {
                    std::fs::create_dir_all(copy_to)
                        .expect("Couldn't create destination directory");
                    let filename = single.file_name().expect("Couldn't extract filename");
                    copy_to = destination_path.join(filename);
                };

                #[expect(
                    clippy::expect_used,
                    reason = "There's no way to continue if copying fails"
                )]
                std::fs::copy(single, copy_to.clone())
                    .expect("Couldn't copy shader to destination");

                tracing::info!("✅ Compiled to: {copy_to:?}");

                if let Some(validation) = maybe_validation {
                    let validation_result = match validation {
                        ValidationOption::Spriv => validate(single, false),
                        ValidationOption::Wgsl => validate(single, true),
                    };
                    if let Err(error) = validation_result {
                        tracing::error!("{error}");
                    }
                }
            }

            #[expect(clippy::unimplemented, reason = "Remove once we support multimodules")]
            spirv_builder::ModuleResult::MultiModule(multi) => {
                tracing::info!("✅ Compile success (multiple module files)");
                for (key, module) in multi {
                    tracing::info!("{key:}: {module:?}");
                }
                unimplemented!("Multimodule support not yet implemented");
            }
        };
    }
}
