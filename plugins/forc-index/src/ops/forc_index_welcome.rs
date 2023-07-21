use crate::cli::InitCommand;
use crate::ops::forc_index_init;
use crate::utils::default_manifest_filename;
use forc_util::{kebab_to_snake_case, validate_name};
use fuel_indexer_lib::manifest::{ContractIds, Manifest};
use inquire::{required, Confirm, Select, Text};
use lazy_static::lazy_static;
use std::collections::HashMap;
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

    let native = !Confirm::new("Would you like to compile your indexer to WebAssembly?")
        .with_default(true)
        .with_help_message(
            "WebAssembly is the recommended execution mode for indexers (default: true)",
        )
        .prompt()?;

    let command = InitCommand {
        name: Some(identifier),
        path: Some(parent_folder),
        namespace,
        native,
        absolute_paths: false,
        verbose: false,
    };
    forc_index_init::init(command, false)?;

    println!(
        "\nThanks! A default indexer has been created at {}.",
        project_dir.display()
    );
    println!("The directory contains a configuration manifest, schema, and source file for your handler code.\n");

    let should_continue = Confirm::new(
        "Would you like to continue to customizing and deploying your indexer?",
    )
    .prompt()?;

    if !should_continue {
        println!("The directory contains a configuration manifest, schema, and source file for your handler code.");
        return Ok(());
    }

    println!(
        "The rest of these fields are optional; feel free to use the default values.\n"
    );

    let abi = Text::new("Path to ABI:")
        .with_default("")
        .with_help_message("Path to the JSON ABI of your contract (default: none)")
        .prompt()?;
    let abi = if abi.is_empty() { None } else { Some(abi) };

    let contract_id = Text::new("Enter a contract ID to subscribe to:")
        .with_default("")
        .with_help_message("An indexer can listen to all contracts or a specific set of contracts. (default: none)")
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
            "The block at which your indexer will start processing information (default: none)",
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
            "The block at which your indexer will stop processing information (default: none)",
        )
        .prompt()?;

    let end_block = if end_block.is_empty() {
        None
    } else {
        Some(end_block.parse::<u64>()?)
    };

    let resumable = Confirm::new("Would you like your indexer to be resumable?")
        .with_default(true)
        .with_help_message("Specifies whether an indexer will automatically sync with the chain upon starting the indexer service (default: true)")
        .prompt()?;

    let metrics_enabled = Confirm::new("Would you like to enable metrics for your indexer?")
        .with_default(false)
        .with_help_message(
            "Enables metrics collection; endpoint will be available at `/api/metrics` (default: false)",
        )
        .prompt()?;

    let manifest_path =
        Path::new(&project_dir).join(default_manifest_filename(&project_name));

    let mut manifest = Manifest::from_file(manifest_path.clone())?;

    if abi.is_some() {
        manifest.set_abi(abi.unwrap());
    }

    if start_block.is_some() {
        manifest.set_start_block(start_block.unwrap());
    }

    if end_block.is_some() {
        manifest.set_end_block(end_block.unwrap());
    }

    manifest.set_contract_id(contract_id);
    manifest.set_fuel_client(fuel_client);
    manifest.set_resumable(resumable);
    manifest.set_metrics(metrics_enabled);

    manifest.write(&manifest_path)?;

    Ok(())
}
