use crate::{cli::BuildCommand, defaults, utils::project_dir_info};
use fuel_indexer_lib::manifest::{Manifest, Module};
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
    time::Duration,
};
use tracing::info;

#[derive(Deserialize)]
#[allow(unused)]
struct Package {
    name: String,
    version: String,
    edition: String,
    publish: bool,
}

#[derive(Deserialize)]
#[allow(unused)]
struct Lib {
    #[serde(alias = "crate-type")]
    crate_type: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[allow(unused)]
struct Config {
    package: Package,
    lib: Option<Lib>,
}

pub fn init(command: BuildCommand) -> anyhow::Result<()> {
    let BuildCommand {
        native,
        path,
        debug,
        locked,
        manifest,
        verbose,
        ..
    } = command;

    let release = !debug;

    let (root_dir, manifest, _index_name) =
        project_dir_info(path.as_ref(), manifest.as_ref())?;

    // Must be in the directory of the index being built
    let cargo_manifest_path = root_dir.join(defaults::CARGO_MANIFEST_FILE_NAME);
    if !cargo_manifest_path.exists() {
        let cargo_manifest_dir = {
            let mut path = cargo_manifest_path.clone();
            path.pop();
            path
        };
        anyhow::bail!(
            "could not find `Cargo.toml` in `{}`",
            cargo_manifest_dir.display()
        );
    }

    let mut file = File::open(&cargo_manifest_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let config: Config = toml::from_str(&content)?;

    let indexer_manifest_path = root_dir.join(manifest);
    let mut manifest = Manifest::from_file(&indexer_manifest_path)?;

    // Rebuild the WASM module even if only the schema has changed.
    crate::ops::utils::ensure_rebuild_if_schema_changed(
        root_dir.as_path(),
        Path::new(manifest.graphql_schema()),
    )?;

    // Construct our build command
    //
    // https://doc.rust-lang.org/cargo/commands/cargo-build.html
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--manifest-path")
        .arg(&cargo_manifest_path);

    if !native {
        cmd.arg("--target").arg(defaults::WASM_TARGET);
    }

    let bool_opts = [
        (release, "--release"),
        (verbose, "--verbose"),
        (locked, "--locked"),
    ];

    for (value, flag) in bool_opts.iter() {
        if *value {
            cmd.arg(flag);
        }
    }

    // Do the build
    if verbose {
        match cmd
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
        {
            Ok(mut proc) => match proc.wait() {
                Ok(s) => {
                    if s.success() {
                        info!("✅ Build succeeded.");
                    } else {
                        anyhow::bail!("❌ Build failed.");
                    }
                }
                Err(e) => {
                    anyhow::bail!("❌ Failed to get ExitStatus of build: {e}.",);
                }
            },
            Err(e) => {
                anyhow::bail!(format!("❌ Build failed: {e}",));
            }
        }
    } else {
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(120));
        pb.set_style(
            ProgressStyle::with_template("{spinner:.blue} {msg}")
                .unwrap()
                .tick_strings(&[
                    "▹▹▹▹▹",
                    "▸▹▹▹▹",
                    "▹▸▹▹▹",
                    "▹▹▸▹▹",
                    "▹▹▹▸▹",
                    "▹▹▹▹▸",
                    "▪▪▪▪▪",
                ]),
        );
        pb.set_message("⏰ Building...");

        match cmd.output() {
            Ok(o) => {
                std::io::stdout()
                    .write_all(&o.stdout)
                    .expect("Failed to write to stdout.");

                match cmd.status() {
                    Ok(s) => {
                        if s.success() {
                            pb.finish_with_message("✅ Build succeeded.");
                        } else {
                            pb.finish_with_message("❌ Build failed.");
                            anyhow::bail!("❌ Failed to build index.");
                        }
                    }
                    Err(e) => {
                        pb.finish_with_message("❌ Build failed.");
                        anyhow::bail!("❌ Failed to determine process exit status: {e}.",);
                    }
                }
            }
            Err(e) => {
                pb.finish_with_message("❌ Build failed.");
                anyhow::bail!(format!("❌ Error: {e}",));
            }
        }
    }

    // Write the build artifacts to the indexer manifest
    if !native {
        let binary = format!("{}.wasm", config.package.name);
        let profile = if release { "release" } else { "debug" };

        let path = path.unwrap_or(".".into());

        let target_dir: std::path::PathBuf =
            crate::ops::utils::cargo_target_dir(path.as_path()).unwrap();

        let abs_artifact_path = target_dir
            .join(defaults::WASM_TARGET)
            .join(profile)
            .join(&binary);

        let rel_artifact_path = Path::new("target")
            .join(defaults::WASM_TARGET)
            .join(profile)
            .join(&binary);

        let abs_wasm = abs_artifact_path.as_path().display().to_string();
        let relative_wasm = rel_artifact_path.as_path().display().to_string();

        manifest.set_module(Module::Wasm(relative_wasm));

        let status = Command::new("wasm-snip")
            .arg(&abs_wasm)
            .arg("-o")
            .arg(&abs_wasm)
            .arg("-p")
            .arg("__wbindgen")
            .spawn()
            .unwrap_or_else(|e| panic!("❌ Failed to spawn wasm-snip process: {e}",))
            .wait()
            .unwrap_or_else(|e| panic!("❌ Failed to finish wasm-snip process: {e}",));

        if !status.success() {
            let code = status.code();
            anyhow::bail!("❌ Failed to execute wasm-snip: (Code: {code:?})",)
        }

        manifest.write(&indexer_manifest_path)?;
    }

    Ok(())
}
