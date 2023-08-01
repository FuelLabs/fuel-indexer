use crate::{cli::NewCommand, defaults, utils::*};
use anyhow::Context;
use forc_util::{kebab_to_snake_case, validate_name};
use std::{fs, path::Path};
use tracing::info;

fn print_welcome_message() {
    let read_the_docs = format!(
        "Read the Docs:\n- {}\n- {}\n- {}\n- {}\n",
        "Fuel Indexer: https://github.com/FuelLabs/fuel-indexer",
        "Fuel Indexer Book: https://fuellabs.github.io/fuel-indexer/latest",
        "Sway Book: https://fuellabs.github.io/sway/latest",
        "Rust SDK Book: https://rust.fuel.network",
    );

    let join_the_community = format!(
        "Join the Community:\n- Follow us {}
- Ask questions in dev-chat on {}",
        "@Fuel: https://twitter.com/fuel_network",
        "Discord: https://discord.com/invite/xfpK4Pe"
    );

    let report_bugs = format!(
        "Report Bugs:\n- {}",
        "Fuel Indexer Issues: https://github.com/FuelLabs/fuel-indexer/issues/new"
    );

    let plugin_msg = r#"Take a quick tour.

`forc index auth`
    Authenticate against an indexer service.
`forc index build`
    Build an indexer.
`forc index check`
    List indexer components.
`forc index deploy`
    Deploy an indexer.
`forc index kill`
    Kill a running Fuel indexer process on a given port.
`forc index new`
    Create a new indexer.
`forc index remove`
    Stop a running indexer.
`forc index start`
    Start a local indexer service.
`forc index status`
    Check the status of an indexer."#;

    let ascii_tag = r#"
███████ ██    ██ ███████ ██          ██ ███    ██ ██████  ███████ ██   ██ ███████ ██████ 
██      ██    ██ ██      ██          ██ ████   ██ ██   ██ ██       ██ ██  ██      ██   ██ 
█████   ██    ██ █████   ██          ██ ██ ██  ██ ██   ██ █████     ███   █████   ██████  
██      ██    ██ ██      ██          ██ ██  ██ ██ ██   ██ ██       ██ ██  ██      ██   ██ 
██       ██████  ███████ ███████     ██ ██   ████ ██████  ███████ ██   ██ ███████ ██   ██ 
                                                                                          

An easy-to-use, flexible indexing service built to go fast. 🚗💨
    "#;

    info!(
        "\n{}\n\n----\n\n{}\n\n{}\n\n{}\n\n{}\n",
        ascii_tag, read_the_docs, join_the_community, report_bugs, plugin_msg
    );
}

pub fn create_indexer(command: NewCommand) -> anyhow::Result<()> {
    let NewCommand {
        name,
        path: project_dir,
        namespace,
        native,
        absolute_paths,
        verbose,
    } = command;

    std::fs::create_dir_all(&project_dir)?;

    if project_dir
        .join(defaults::CARGO_MANIFEST_FILE_NAME)
        .exists()
    {
        anyhow::bail!(
            "❌ '{}' already includes a Cargo.toml file.",
            project_dir.display()
        );
    }

    if verbose {
        info!(
            "\nUsing project directory at {}",
            project_dir.canonicalize()?.display()
        );
    }

    let project_name = match name {
        Some(name) => name,
        None => project_dir
            .file_stem()
            .context("❌ Failed to infer project name from directory name.")?
            .to_string_lossy()
            .into_owned(),
    };

    // Indexer expects underscores not dashes
    let project_name = kebab_to_snake_case(&project_name);

    validate_name(&project_name, "project name")?;

    // Make a new directory for the project
    fs::create_dir_all(Path::new(&project_dir).join("src"))?;

    let default_toml = if native {
        defaults::default_native_indexer_cargo_toml(&project_name)
    } else {
        defaults::default_indexer_cargo_toml(&project_name)
    };

    // Write index Cargo manifest
    fs::write(
        Path::new(&project_dir).join(defaults::CARGO_MANIFEST_FILE_NAME),
        default_toml,
    )?;

    let proj_abspath = if absolute_paths {
        Some(fs::canonicalize(Path::new(&project_dir))?)
    } else {
        None
    };

    // If not supplied, set namespace to system username
    let namespace = if let Some(ns) = namespace {
        ns
    } else {
        whoami::username()
    };

    let manifest_filename = default_manifest_filename(&project_name);
    let schema_filename = default_schema_filename(&project_name);

    // Write index manifest
    fs::write(
        Path::new(&project_dir).join(&manifest_filename),
        defaults::default_indexer_manifest(
            &namespace,
            &schema_filename,
            &project_name,
            proj_abspath.as_ref(),
            native,
        ),
    )?;

    // Write index schema
    fs::create_dir_all(Path::new(&project_dir).join("schema"))?;
    fs::write(
        Path::new(&project_dir).join("schema").join(schema_filename),
        defaults::default_indexer_schema(),
    )?;

    // What content are we writing?
    let (filename, content) = if native {
        (
            defaults::INDEXER_BINARY_FILENAME,
            defaults::default_indexer_binary(
                &project_name,
                &manifest_filename,
                proj_abspath.as_ref(),
            ),
        )
    } else {
        (
            defaults::INDEXER_LIB_FILENAME,
            defaults::default_indexer_lib(
                &project_name,
                &manifest_filename,
                proj_abspath.as_ref(),
            ),
        )
    };

    // Write indexer file
    fs::write(Path::new(&project_dir).join("src").join(filename), content)?;

    // Write cargo config with WASM target
    if !native {
        fs::create_dir_all(
            Path::new(&project_dir).join(defaults::CARGO_CONFIG_DIR_NAME),
        )?;
        let _ = fs::write(
            Path::new(&project_dir)
                .join(defaults::CARGO_CONFIG_DIR_NAME)
                .join(defaults::CARGO_CONFIG_FILENAME),
            defaults::default_cargo_config(),
        );
    }

    if verbose {
        info!("\n✅ Successfully created indexer {project_name}");
    } else {
        info!("\n✅ Successfully created indexer");
    }
    Ok(())
}

/// Execute the command for `forc_index_new`.
pub fn init(command: NewCommand) -> anyhow::Result<()> {
    create_indexer(command)?;
    print_welcome_message();
    Ok(())
}
