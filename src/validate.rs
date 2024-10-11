//! Validate a SPIR-V ".spv" binary.
//! Thanks to @schell for the [code](https://github.com/Rust-GPU/rust-gpu/discussions/23#discussioncomment-10912823)

use anyhow::Context;
use naga::valid::ValidationFlags;

/// Validation entry point.
pub fn validate(path: &std::path::PathBuf, is_validate_wgsl: bool) -> anyhow::Result<()> {
    let (spirv_module, spirv_info, is_spirv_valid) = validate_spirv(path)?;

    if is_validate_wgsl {
        let wgsl_module = create_wgsl(path, &spirv_module, &spirv_info)?;
        validate_wgsl(&wgsl_module)?;
    }

    if !is_spirv_valid {
        anyhow::bail!("SPIR-V validation error");
    }

    Ok(())
}

/// Validate the SPIR-V binary.
fn validate_spirv(
    path: &std::path::PathBuf,
) -> anyhow::Result<(naga::Module, naga::valid::ModuleInfo, bool)> {
    let is_spirv_valid;

    tracing::info!("validating source");
    tracing::info!("  reading '{}'", path.display());

    let bytes = std::fs::read(path)?;
    tracing::info!("  {:0.2}k bytes read", bytes.len() as f32 / 1000.0);

    let opts = naga::front::spv::Options::default();
    let spirv_module = match naga::front::spv::parse_u8_slice(&bytes, &opts) {
        Ok(module) => module,
        Err(error) => anyhow::bail!(error),
    };
    tracing::info!("  SPIR-V parsed");

    let mut spirv_validator = naga::valid::Validator::new(
        ValidationFlags::default(),
        naga::valid::Capabilities::empty(),
    );
    let spirv_info = match spirv_validator.validate(&spirv_module) {
        Ok(info) => {
            is_spirv_valid = true;
            tracing::info!("  SPIR-V validated");
            info
        }
        Err(error) => {
            tracing::error!("{}", error.emit_to_string(""));
            is_spirv_valid = false;

            tracing::info!(" re-validating without any validation rules");
            let mut validator = naga::valid::Validator::new(
                ValidationFlags::empty(),
                naga::valid::Capabilities::empty(),
            );
            match validator.validate(&spirv_module) {
                Ok(info) => info,
                Err(revalidation_error) => {
                    tracing::error!("SPIR-V revalidation (with zero validation flags) also failed");
                    tracing::error!("{}", error.emit_to_string(""));
                    anyhow::bail!(revalidation_error)
                }
            }
        }
    };

    Ok((spirv_module, spirv_info, is_spirv_valid))
}

/// Convert the SPIR-V module to WGSL using `naga`.
fn create_wgsl(
    path: &std::path::Path,
    spirv_module: &naga::Module,
    spirv_info: &naga::valid::ModuleInfo,
) -> anyhow::Result<String> {
    let wgsl = naga::back::wgsl::write_string(
        spirv_module,
        spirv_info,
        naga::back::wgsl::WriterFlags::empty(),
    )?;
    tracing::info!("  output WGSL generated");

    let print_var_name = path
        .file_stem()
        .context("Couldn't get SPIR-V path file stem")?
        .to_str()
        .context("Couldn't get SPIR-V path to string")?
        .replace('-', "_");

    let dir = std::env::temp_dir();
    std::fs::create_dir_all(&dir)?;
    let output_path = dir.join(print_var_name).with_extension("wgsl");
    tracing::info!("writing WGSL to '{}'", output_path.display());

    std::fs::write(&output_path, &wgsl)?;
    tracing::info!("  wrote generated WGSL to {}", output_path.display());

    Ok(wgsl)
}

/// Validate the cross-compiled WGSL version of the SPIR-V module.
fn validate_wgsl(wgsl_string: &str) -> anyhow::Result<()> {
    let wgsl_module = match naga::front::wgsl::parse_str(wgsl_string) {
        Ok(module) => module,
        Err(error) => {
            anyhow::bail!("{}", error.emit_to_string(wgsl_string));
        }
    };
    tracing::info!("  output WGSL parsed");
    let mut wgsl_validator = naga::valid::Validator::new(
        ValidationFlags::default(),
        naga::valid::Capabilities::empty(),
    );
    let _info = match wgsl_validator.validate(&wgsl_module) {
        Ok(info) => info,
        Err(error) => {
            anyhow::bail!("{}", error.emit_to_string(wgsl_string));
        }
    };
    tracing::info!("  wgsl output validated");

    Ok(())
}
