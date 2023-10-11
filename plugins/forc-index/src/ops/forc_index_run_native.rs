use crate::{
    cli::{BuildCommand, RunNativeCommand},
    commands::build,
    defaults,
    utils::*,
};
use fuel_indexer_lib::manifest::Manifest;
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};
use tracing::info;

pub async fn init(command: RunNativeCommand) -> anyhow::Result<()> {
    let RunNativeCommand {
        path,
        debug,
        locked,
        manifest: mani_path,
        verbose,
        skip_build,
        bin,
        args,
        ..
    } = command;

    if !skip_build {
        build::exec(BuildCommand {
            manifest: mani_path.clone(),
            path: path.clone(),
            debug,
            verbose,
            locked,
            native: true,
        })?;
    }

    let release = !debug;

    let (root_dir, manifest, indexer_name) =
        project_dir_info(path.as_ref(), mani_path.as_ref())?;

    // Must be in the directory of the indexer being built
    let cargo_manifest_path = root_dir.join(defaults::CARGO_MANIFEST_FILE_NAME);
    if !cargo_manifest_path.exists() {
        let cargo_manifest_dir = {
            let mut path = cargo_manifest_path;
            path.pop();
            path
        };
        anyhow::bail!(
            "could not find `Cargo.toml` in `{}`",
            cargo_manifest_dir.display()
        );
    }

    let current_dir = std::env::current_dir()?;

    let path = path.unwrap_or(current_dir);

    let mut file = File::open(&cargo_manifest_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let indexer_manifest_path = root_dir.join(manifest);
    let manifest = Manifest::from_file(&indexer_manifest_path)?;

    let workspace_root: PathBuf = cargo_workspace_root_dir(path.as_path()).unwrap();

    let manifest_schema_file = Path::new(&workspace_root).join(manifest.graphql_schema());

    let binpath = bin
        .unwrap_or_else(|| {
            let dir = if release { "release" } else { "debug" };
            let name = dasherize_to_underscore(&indexer_name);
            workspace_root.join("target").join(dir).join(name)
        })
        .display()
        .to_string();

    // Rebuild the WASM module even if only the schema has changed.
    ensure_rebuild_if_schema_or_manifest_changed(
        root_dir.as_path(),
        Path::new(manifest_schema_file.as_path()),
        indexer_manifest_path.as_path(),
        manifest.execution_source(),
    )?;

    let mut cmd = Command::new(binpath);
    cmd.arg("--manifest").arg(&indexer_manifest_path);

    for arg in args {
        cmd.arg(arg);
    }

    if verbose {
        info!("{cmd:?}")
    }

    match cmd.spawn() {
        Ok(child) => {
            let pid = child.id();
            info!("✅ Successfully started the indexer service at PID {pid}");
        }
        Err(e) => panic!("❌ Failed to spawn fuel-indexer child process: {e:?}."),
    }

    Ok(())
}
