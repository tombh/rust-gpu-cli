use std::{borrow::Borrow, path::PathBuf, str::FromStr};

#[derive(Debug, Clone, clap::Parser)]
#[command(author, version, about, long_about = None)]
pub struct ShaderBuilder {
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

    /// Enables the provided SPIR-V capability.
    #[arg(long, value_parser=Self::spirv_capability)]
    capability: Vec<spirv_builder::Capability>,

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
}

impl ShaderBuilder {
    /// Clap value parser for `SpirvMetadata`.
    fn spirv_metadata(s: &str) -> Result<spirv_builder::SpirvMetadata, clap::Error> {
        match s {
            "none" => Ok(spirv_builder::SpirvMetadata::None),
            "name-variables" => Ok(spirv_builder::SpirvMetadata::NameVariables),
            "full" => Ok(spirv_builder::SpirvMetadata::Full),
            _ => Err(clap::Error::new(clap::error::ErrorKind::InvalidValue)),
        }
    }

    /// Clap value parser for `Capability`.
    fn spirv_capability(s: &str) -> Result<spirv_builder::Capability, clap::Error> {
        match spirv_builder::Capability::from_str(s) {
            Ok(capability) => Ok(capability),
            Err(_) => Err(clap::Error::new(clap::error::ErrorKind::InvalidValue)),
        }
    }

    fn make_builder(&self) -> spirv_builder::SpirvBuilder {
        // Hack(from `rust-gpu`):
        // `spirv_builder` builds into a custom directory if running under cargo, to not
        // deadlock, and the default target directory if not. However, packages like `proc-macro2`
        // have different configurations when being built here vs. when building
        // `rustc_codegen_spirv`` normally, so we *want* to build into a separate target directory, to
        // not have to rebuild half the crate graph every time we run. So, pretend we're running
        // under Cargo by setting these environment variables.
        std::env::set_var("OUT_DIR", env!("OUT_DIR"));
        std::env::set_var("PROFILE", env!("PROFILE"));

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

        builder
    }

    /// Starts watching a shader directory and compiles on changes
    pub fn build_shader_daemon(
        &self,
    ) -> Result<spirv_builder::CompileResult, spirv_builder::SpirvBuilderError> {
        tracing::info!("Starting daemon");

        let builder = self.make_builder();
        let source = self.path_to_crate.clone();
        let is_custom_output_path = self.output_path.is_some();
        let destination = match self.output_path.borrow() {
            Some(path) => path.clone(),
            None => source.join("compiled"),
        };

        builder.watch(move |compile_result| {
            match &compile_result.module {
                spirv_builder::ModuleResult::SingleModule(single) => {
                    let mut copy_to = destination.clone();
                    if !is_custom_output_path {
                        std::fs::create_dir_all(copy_to)
                            .expect("Couldn't create destination directory");
                        let filename = single.file_name().expect("Couldn't extract filename");
                        copy_to = destination.join(filename);
                    };
                    std::fs::copy(single, copy_to.clone())
                        .expect("Couldn't copy shader to destination");
                    tracing::info!("✅ Compiled to: {copy_to:?}");
                }

                spirv_builder::ModuleResult::MultiModule(multi) => {
                    tracing::info!("✅ Compile success (multiple module files)");
                    for (k, module) in multi {
                        tracing::info!("{k:}: {module:?}");
                    }
                    unimplemented!("Multimodule support not yet implemented");
                }
            };
        })
    }
}
