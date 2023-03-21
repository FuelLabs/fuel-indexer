use crate::{
    cli::BuildCommand,
    utils::{defaults, project_dir_info},
};
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
        target: target_triple,
        native,
        path,
        verbose,
        profile,
        release,
        locked,
        manifest,
        output_dir_root,
    } = command;

    let (root_dir, manifest, _index_name) =
        project_dir_info(path.as_ref(), manifest.as_ref())?;

    // Must be in the directory of the index being built
    let cargo_manifest_path = root_dir.join(defaults::CARGO_MANIFEST_FILE_NAME);
    if !cargo_manifest_path.exists() {
        let manifest_path = cargo_manifest_path.into_os_string();
        anyhow::bail!(
            "❌ `forc index build` must be run from inside the directory of the index being built. Cargo manifest file expected at '{manifest_path:?}'",
        );
    }

    let mut file = File::open(&cargo_manifest_path)?;
    println!("file: {:?}", file);
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let config: Config = toml::from_str(&content)?;

    let index_manifest_path = root_dir.join(manifest);
    let mut manifest = Manifest::from_file(&index_manifest_path)?;

    // Construct our build command
    //
    // https://doc.rust-lang.org/cargo/commands/cargo-build.html
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--manifest-path")
        .arg(&cargo_manifest_path);

    let optional_opts = [(target_triple.clone(), "--target"), (profile, "--profile")];
    let bool_opts = [
        (release, "--release"),
        (verbose, "--verbose"),
        (locked, "--locked"),
    ];

    for (value, flag) in optional_opts.iter() {
        if let Some(v) = value {
            cmd.arg(flag).arg(v);
        }
    }

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

    // Write the build artifact to the index manifest
    if !native {
        let binary = format!("{}.wasm", config.package.name);
        let profile = if release { "release" } else { "debug" };
        let output_dir_root = output_dir_root.unwrap_or(root_dir);
        let abs_artifact_path = output_dir_root
            .join("target")
            .join(
                target_triple
                    .clone()
                    .unwrap_or_else(|| defaults::INDEX_TARGET.into()),
            )
            .join(profile)
            .join(&binary);

        let target_root =
            std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());
        let target_root = Path::new(&target_root);

        let rel_artifact_path = target_root
            .join(target_triple.unwrap_or_else(|| defaults::INDEX_TARGET.into()))
            .join(profile)
            .join(&binary);

        let abs_wasm = abs_artifact_path.as_path().display().to_string();
        let relative_wasm = rel_artifact_path.as_path().display().to_string();

        manifest.module = Module::Wasm(relative_wasm);

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

        manifest.write_to(&index_manifest_path)?;
    }

    Ok(())
}
