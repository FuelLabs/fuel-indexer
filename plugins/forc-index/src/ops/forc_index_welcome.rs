use crate::defaults;
use crate::utils::{default_manifest_filename, default_schema_filename};
use forc_util::{kebab_to_snake_case, validate_name};
use fuel_indexer_lib::manifest::{ContractIds, Manifest};
use inquire::{required, Confirm, Select, Text};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

lazy_static! {
    pub static ref AVAILABLE_NETWORKS: HashMap<&'static str, &'static str> =
        HashMap::from([
            ("Local Node", "127.0.0.1:4000"),
            ("Beta-3", "beta-3.fuel.network:80"),
        ]);
}

pub async fn init() -> anyhow::Result<()> {
    println!("Let's create an indexer! First, please fill in the following fields.\n");
    let namespace = Text::new("Namespace:")
        .with_validator(required!())
        .with_help_message("The topmost organizational level of your indexer")
        .prompt()?;
    let identifier = Text::new("Identifier:")
        .with_validator(required!())
        .with_help_message("A unique identifer for your indexer")
        .prompt()?;

    let project_name = kebab_to_snake_case(&identifier);
    validate_name(&project_name, "Project name")?;

    let parent_folder =
        if Confirm::new("Do you want to save your indexer in the current directory?")
            .prompt()?
        {
            std::env::current_dir()?
        } else {
            let path_str = Text::new("Enter the path to an existing directory:")
                .with_help_message("Your indexer will be saved inside of this directory")
                .prompt()?;
            std::fs::canonicalize(PathBuf::from_str(&path_str)?)?
        };

    let project_dir = parent_folder.join(identifier.clone());

    println!("\nThanks! The rest of these fields are optional; feel free to use the default values.\n");

    let abi = Text::new("Path to ABI:").with_default("").prompt()?;
    let abi = if abi.is_empty() { None } else { Some(abi) };

    let contract_id = Text::new("Enter a contract ID to subscribe to:")
        .with_default("")
        .prompt()?;

    let contract_id = if contract_id.is_empty() {
        ContractIds::Single(None)
    } else {
        ContractIds::from_str(&contract_id).unwrap()
    };

    let fuel_client_key = Select::new(
        "What network should your indexer connect to?",
        AVAILABLE_NETWORKS
            .keys()
            .map(ToOwned::to_owned)
            .map(String::from)
            .collect::<Vec<String>>(),
    )
    .with_starting_cursor(0)
    .prompt()?;

    let fuel_client = AVAILABLE_NETWORKS
        .get(&fuel_client_key.as_str())
        .unwrap()
        .to_string();

    let start_block = Text::new("Enter a start block:")
        .with_default("")
        .with_help_message(
            "The block at which your indexer will start processing information",
        )
        .prompt()?;

    let start_block = if start_block.is_empty() {
        None
    } else {
        Some(start_block.parse::<u64>()?)
    };

    let end_block = Text::new("Enter a end block:")
        .with_default("")
        .with_help_message(
            "The block at which your indexer will stop processing information",
        )
        .prompt()?;

    let end_block = if end_block.is_empty() {
        None
    } else {
        Some(end_block.parse::<u64>()?)
    };

    let native = !Confirm::new("Would you like to compile your indexer to WebAssembly?")
        .with_default(true)
        .with_help_message("WebAssembly is the recommended execution mode for indexers")
        .prompt()?;

    let resumable = Confirm::new("Would you like your indexer to be resumable?")
        .with_default(false)
        .with_help_message("Specifies whether an indexer will automatically sync with the chain upon starting the indexer service")
        .prompt()?;

    let metrics = Confirm::new("Would you like to enable metrics for your indexer?")
        .with_default(false)
        .with_help_message(
            "Enables metrics collection; endpoint will be available at `/api/metrics`",
        )
        .prompt()?;

    let manifest = Manifest {
        namespace,
        identifier,
        graphql_schema: default_schema_filename(&project_name),
        contract_id,
        abi,
        fuel_client: Some(fuel_client),
        module: fuel_indexer_lib::manifest::Module::Wasm("".to_string()),
        metrics: Some(metrics),
        start_block,
        end_block,
        resumable: Some(resumable),
    };

    fs::create_dir_all(Path::new(&project_dir).join("src"))?;

    let default_toml = defaults::default_indexer_cargo_toml(&project_name);

    fs::write(
        Path::new(&project_dir).join(defaults::CARGO_MANIFEST_FILE_NAME),
        default_toml,
    )
    .expect("Failed to write Cargo manifest");

    let manifest_filename = default_manifest_filename(&project_name);
    let schema_filename = default_schema_filename(&project_name);

    manifest.write(&project_dir.join(manifest_filename.clone()))?;

    fs::create_dir_all(Path::new(&project_dir).join("schema"))?;
    fs::write(
        Path::new(&project_dir).join("schema").join(schema_filename),
        defaults::default_indexer_schema(),
    )
    .expect("Failed to write GraphQL schema");

    let (filename, content) = if native {
        (
            defaults::INDEXER_BINARY_FILENAME,
            defaults::default_indexer_binary(&project_name, &manifest_filename, None),
        )
    } else {
        (
            defaults::INDEXER_LIB_FILENAME,
            defaults::default_indexer_lib(&project_name, &manifest_filename, None),
        )
    };

    fs::write(Path::new(&project_dir).join("src").join(filename), content)
        .expect("Failed to write indexer source file");

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

    println!(
        "\nYour indexer has been created at {}",
        project_dir.display()
    );

    Ok(())
}
