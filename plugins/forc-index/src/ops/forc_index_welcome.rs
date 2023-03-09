use crate::ops::{
    forc_index_build::init as build, forc_index_deploy::init as deploy,
    forc_index_init::create_indexer as create, forc_index_start::init as start,
};
use crate::{
    cli::{BuildCommand, DeployCommand, InitCommand, StartCommand, WelcomeCommand},
    utils::defaults,
};
use owo_colors::OwoColorize;
use rand::Rng;
use std::{
    io::{self, Write},
    thread, time,
};
use tracing::info;

enum Network {
    Local,
    Testnet,
}

const TITLE: &str = "

███████╗██╗░░░██╗███████╗██╗░░░░░  ██╗███╗░░██╗██████╗░███████╗██╗░░██╗███████╗██████╗░
██╔════╝██║░░░██║██╔════╝██║░░░░░  ██║████╗░██║██╔══██╗██╔════╝╚██╗██╔╝██╔════╝██╔══██╗
█████╗░░██║░░░██║█████╗░░██║░░░░░  ██║██╔██╗██║██║░░██║█████╗░░░╚███╔╝░█████╗░░██████╔╝
██╔══╝░░██║░░░██║██╔══╝░░██║░░░░░  ██║██║╚████║██║░░██║██╔══╝░░░██╔██╗░██╔══╝░░██╔══██╗
██║░░░░░╚██████╔╝███████╗███████╗  ██║██║░╚███║██████╔╝███████╗██╔╝╚██╗███████╗██║░░██║
╚═╝░░░░░░╚═════╝░╚══════╝╚══════╝  ╚═╝╚═╝░░╚══╝╚═════╝░╚══════╝╚═╝░░╚═╝╚══════╝╚═╝░░╚═╝
";

const WELCOME_MANIFEST: &str = "welcome.manifest.yaml";
const WASM_TARGET: &str = "wasm32-unknown-unknown";

pub async fn init(command: WelcomeCommand) -> anyhow::Result<()> {
    for line in TITLE.lines() {
        println!("{}", line.trim().bright_cyan());
        thread::sleep(time::Duration::from_millis(50));
    }

    humanize_message("\n Welcome to the Fuel Indexer CLI 🚀".to_string());
    thread::sleep(time::Duration::from_millis(500));
    humanize_message("\n This tool will help you understand how to create and deploy an index on the Fuel blockchain.".to_string());
    thread::sleep(time::Duration::from_millis(500));
    humanize_message("\n Let's get started!".to_string());
    thread::sleep(time::Duration::from_millis(500));
    humanize_message("\n First, we'll create a new index.".to_string());
    thread::sleep(time::Duration::from_millis(500));
    humanize_message("\n Would you like to create the default index? (Y/n)".to_string());

    let mut input = String::new();
    input = process_std(input);

    match input.trim().to_lowercase().as_str() {
        "y" | "yes" => {
            info!("\n Creating the default index... 🔥");
            create(InitCommand {
                name: Some("welcome".to_string()),
                path: Some(std::path::PathBuf::from(".")),
                namespace: "default".to_string(),
                native: false,
                absolute_paths: true,
            })?;

            info!("\n✅ Indexer project initialized. Manifest file created.");
            info!("\n Start the indexer and deploy the index? (Y/n)");

            input = process_std(input);
            match input.trim().to_lowercase().as_str() {
                "y" | "yes" => {
                    info!("\n Connect to which network? 🤔 \n1. Local node\n2. Testnet");
                    input = process_std(input);

                    match input.trim().to_lowercase().as_str() {
                        "1" => {
                            start(init_start(Network::Local))?;
                            build(init_build())?;
                            deploy(init_deploy())?;
                        }
                        "2" => {
                            start(init_start(Network::Testnet))?;
                            build(init_build())?;
                            deploy(init_deploy())?;
                        }
                        _ => {
                            println!("Invalid input. Please enter 1 or 2");
                        }
                    }
                }
                "n" | "no" => {
                    println!("Skipping indexer deployment...");
                    std::process::exit(0);
                }
                _ => {
                    println!("Invalid input. Please enter Y or n");
                }
            }
        }
        "n" | "no" => {
            info!("\n Create a name for your index");
            input = process_std(input);
            let name = input.trim().to_lowercase();
        }
        _ => {
            println!("Invalid input. Please enter Y or n");
        }
    }
    Ok(())
}

fn humanize_message(output: String) {
    for c in output.chars() {
        info!("{}", c);
        io::stdout().flush().unwrap();
        let sleep_time = rand::thread_rng().gen_range(20..77);
        thread::sleep(time::Duration::from_millis(sleep_time));
    }
}

fn process_std(mut input: String) -> String {
    input.clear();
    io::stdout().flush().expect("failed to flush stdout");
    io::stdin()
        .read_line(&mut input)
        .expect("failed to read from stdin");
    input
}

fn init_start(on_network: Network) -> StartCommand {
    info!("Starting indexer...");
    let mut start_command = StartCommand {
        log_level: "info".to_string(),
        config: None,
        manifest: Some(std::path::PathBuf::from(".")),
        fuel_node_host: String::new(),
        fuel_node_port: String::new(),
        graphql_api_host: String::new(),
        graphql_api_port: String::new(),
        database: defaults::DATABASE.to_string(),
        max_body: defaults::MAX_BODY.to_string(),
        postgres_user: None,
        postgres_database: None,
        postgres_password: None,
        postgres_host: None,
        postgres_port: None,
        run_migrations: true,
        metrics: false,
        stop_idle_indexers: true,
    };
    match on_network {
        Network::Local => {
            start_command.fuel_node_host = "http://127.0.0.1:29987".to_string();
            start_command.fuel_node_port = "29987".to_string();
            start_command.graphql_api_host = defaults::GRAPHQL_API_HOST.to_string();
            start_command.graphql_api_port = defaults::GRAPHQL_API_PORT.to_string();
        }
        Network::Testnet => {
            start_command.fuel_node_host =
                "https://node-beta-2.fuel.network/graphql".to_string();
            start_command.fuel_node_port = "4000".to_string();
            start_command.graphql_api_host = "node-beta-2.fuel.network".to_string();
            start_command.graphql_api_port = "80".to_string();
        }
    }
    start_command
}

fn init_build() -> BuildCommand {
    info!("Building indexer for local node deployment...");
    BuildCommand {
        manifest: Some(WELCOME_MANIFEST.to_string()),
        path: None,
        target: Some(WASM_TARGET.to_string()),
        release: true,
        profile: Some("release".to_string()),
        verbose: false,
        locked: false,
        native: false,
        output_dir_root: Some(std::path::PathBuf::from(".")),
    }
}

fn init_deploy() -> DeployCommand {
    info!("Deploying indexer to local node...");
    DeployCommand {
        url: "http://127.0.0.1:29987".to_string(),
        manifest: Some(WELCOME_MANIFEST.to_string()),
        path: None,
        auth: Some("".to_string()),
        target: Some(WASM_TARGET.to_string()),
        release: true,
        profile: Some("release".to_string()),
        verbose: false,
        locked: false,
        native: false,
        output_dir_root: Some(std::path::PathBuf::from(".")),
    }
}
