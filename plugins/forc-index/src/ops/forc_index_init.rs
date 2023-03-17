use crate::{
    cli::InitCommand,
    utils::{default_manifest_filename, default_schema_filename, defaults},
};
use anyhow::Context;
use forc_util::{kebab_to_snake_case, validate_name};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tracing::info;

fn print_welcome_message() {
    let read_the_docs = format!(
        "Read the Docs:\n- {}\n- {}\n- {}\n- {}\n",
        "Fuel Indexer: https://github.com/FuelLabs/fuel-indexer",
        "Fuel Indexer Book: https://fuellabs.github.io/fuel-indexer/latest",
        "Sway Book: https://fuellabs.github.io/sway/latest",
        "Rust SDK Book: https://fuellabs.github.io/fuels-rs/latest",
    );

    let join_the_community = format!(
        "Join the Community:\n- Follow us {}
- Ask questions in dev-chat on {}",
        "@SwayLang: https://twitter.com/fuellabs_",
        "Discord: https://discord.com/invite/xfpK4Pe"
    );

    let report_bugs = format!(
        "Report Bugs:\n- {}",
        "Fuel Indexer Issues: https://github.com/FuelLabs/fuel-indexer/issues/new"
    );

    let plugin_msg = r#"Take a quick tour.

`forc index check`
    List indexer components.
`forc index new`
    Create a new index.
`forc index init`
    Create a new index in an existing directory.
`forc index start`
    Start a local indexer service.
`forc index build`
    Build your index.
`forc index deploy`
    Deploy your index.
`forc index remove`
    Stop a running index.
`forc index revert`
    Revert a deployed indexer."#;

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

pub fn init(command: InitCommand) -> anyhow::Result<()> {
    let InitCommand {
        name,
        path,
        namespace,
        native,
        absolute_paths,
        verbose,
    } = command;

    let project_dir = match &path {
        Some(p) => PathBuf::from(p),
        None => std::env::current_dir()
            .context("❌ Failed to get current directory for forc index init.")?,
    };

    if !project_dir.is_dir() {
        anyhow::bail!("❌ '{}' is not a valid directory.", project_dir.display());
    }

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
        defaults::default_native_index_cargo_toml(&project_name)
    } else {
        defaults::default_index_cargo_toml(&project_name)
    };

    // Write index Cargo manifest
    fs::write(
        Path::new(&project_dir).join(defaults::CARGO_MANIFEST_FILE_NAME),
        default_toml,
    )
    .unwrap();

    let proj_abspath = if absolute_paths {
        Some(fs::canonicalize(Path::new(&project_dir))?)
    } else {
        None
    };

    let manifest_filename = default_manifest_filename(&project_name);
    let schema_filename = default_schema_filename(&project_name);

    // Write index manifest
    fs::write(
        Path::new(&project_dir).join(&manifest_filename),
        defaults::default_index_manifest(
            &namespace,
            &schema_filename,
            &project_name,
            proj_abspath.as_ref(),
        ),
    )?;

    // Write index schema
    fs::create_dir_all(Path::new(&project_dir).join("schema"))?;
    fs::write(
        Path::new(&project_dir).join("schema").join(schema_filename),
        defaults::default_index_schema(),
    )
    .unwrap();

    // Write lib file
    fs::write(
        Path::new(&project_dir)
            .join("src")
            .join(defaults::INDEX_LIB_FILENAME),
        defaults::default_index_lib(
            &project_name,
            &manifest_filename,
            proj_abspath.as_ref(),
        ),
    )
    .unwrap();

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
        info!("\n✅ Successfully created index {project_name}");
    } else {
        info!("\n✅ Successfully created index");
    }

    print_welcome_message();
    Ok(())
}
