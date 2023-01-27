use crate::{
    cli::InitCommand,
    utils::{dasherize_to_underscore, defaults},
};
use anyhow::Context;
use forc_util::validate_name;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tracing::{debug, info};

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
    Stop a running index."#;

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
    let project_dir = match &command.path {
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

    debug!(
        "\nUsing project directory at {}",
        project_dir.canonicalize()?.display()
    );

    let project_name = match command.name {
        Some(name) => name,
        None => project_dir
            .file_stem()
            .context("❌ Failed to infer project name from directory name.")?
            .to_string_lossy()
            .into_owned(),
    };

    // Indexer expects underscores not dashes
    let project_name = dasherize_to_underscore(&project_name);

    validate_name(&project_name, "project name")?;

    // Make a new directory for the project
    fs::create_dir_all(Path::new(&project_dir).join("src"))?;

    let default_toml = if command.native {
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

    // Write index manifest
    let manifest_filename = format!("{project_name}.manifest.yaml",);
    fs::write(
        Path::new(&project_dir).join(&manifest_filename),
        defaults::default_index_manifest(
            &command.namespace,
            &project_name,
            fs::canonicalize(Path::new(&project_dir))?.to_str().unwrap(),
        ),
    )
    .unwrap();

    // Write index schema
    let schema_filename = format!("{}.schema.graphql", &project_name);
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
            fs::canonicalize(Path::new(&project_dir))?.to_str().unwrap(),
        ),
    )
    .unwrap();

    // Write cargo config with WASM target
    if !command.native {
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

    debug!("\n✅ nSuccessfully created index {project_name}");

    print_welcome_message();

    Ok(())
}
