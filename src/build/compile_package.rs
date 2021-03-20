use crate::{
    build::{Origin, PackageCompiler},
    fs::{self, FileSystemAccessor, FileSystemWriter},
    metadata,
    type_::Module,
    CompilePackage, Result,
};
use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
};

pub type Manifests = HashMap<String, (Origin, Module)>;

pub fn command(options: CompilePackage) -> Result<()> {
    let mut type_manifests = load_libraries(options.libraries.as_slice())?;
    let mut defined_modules = HashMap::new();
    let mut warnings = Vec::new();

    tracing::info!("Compiling package");

    let package = options
        .into_package_compiler_options()
        .into_compiler(FileSystemAccessor::new())?
        .write_metadata(true)
        .compile(&mut warnings, &mut type_manifests, &mut defined_modules)?;

    // Print warnings
    for warning in warnings {
        warning.pretty_print();
    }

    // TODO: Support --warnings-as-errors
    // TODO: Create a Warnings struct to wrap up this functionality

    Ok(())
}

fn load_libraries(libs: &[PathBuf]) -> Result<Manifests> {
    tracing::info!("Reading precompiled module metadata files");
    let mut manifests = HashMap::with_capacity(libs.len() * 10);
    for lib in libs {
        for module in fs::gleam_modules_metadata_paths(lib)? {
            let reader = fs::buffered_reader(module)?;
            let module = metadata::ModuleDecoder::new().read(reader)?;
            let _ = manifests.insert(module.name.join("/"), (Origin::Src, module));
        }
    }
    Ok(manifests)
}
