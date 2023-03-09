use crate::ops::{
    forc_index_build::init as build, forc_index_deploy::init as deploy,
    forc_index_init::create_indexer as create, forc_index_start::init as start,
};
use crate::{
    cli::{BuildCommand, DeployCommand, InitCommand, StartCommand, WelcomeCommand},
    utils::defaults,
};
use forc_util::{kebab_to_snake_case, validate_name};
use owo_colors::OwoColorize;
use rand::{thread_rng, Rng};
use std::fs;
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

const WELCOME_MANIFEST_PATH: &str = "welcome.manifest.yaml";
const WASM_TARGET: &str = "wasm32-unknown-unknown";
const PROJECT_INITIALIZED: &str =
    "\n Indexer project initialized. Manifest file created. ✅";
const DEPLOY_QUESTION: &str = "\n Start the indexer and deploy the index? (Y/n) \n > ";

pub async fn init(command: WelcomeCommand) -> anyhow::Result<()> {
    for line in TITLE.lines() {
        println!("{}", line.trim().bright_cyan());
        thread::sleep(time::Duration::from_millis(50));
    }

    let WelcomeCommand { greeter } = command;
    if greeter {
        render_greeter();
    }

    humanize_message(
        "\n Would you like to create the default index? (Y/n) \n > ".to_string(),
    );

    let mut input = String::new();

    input = process_std(input);
    match input.trim().to_lowercase().as_str() {
        "y" | "yes" => {
            humanize_message("\n Creating the default index... ⚙️".to_string());
            create(InitCommand {
                name: Some("welcome".to_string()),
                path: Some(std::path::PathBuf::from(".")),
                namespace: "default".to_string(),
                native: false,
                absolute_paths: true,
            })?;
            humanize_message(PROJECT_INITIALIZED.to_string());
            humanize_message(DEPLOY_QUESTION.to_string());

            input = process_std(input);
            deploy_to_network(input, WELCOME_MANIFEST_PATH.to_string())?;
        }
        "n" | "no" => {
            humanize_message(
                "\n Ok! Let's create a namespace for your custom index.".to_string(),
            );
            humanize_message(
                "\n Enter a namespace for your index below \n > ".to_string(),
            );
            input = process_std(input);
            let namespace = input.trim().to_lowercase();
            humanize_message(
                "\n Great, now create an identifier for your custom index. \n > "
                    .to_string(),
            );

            input = process_std(input);
            let mut identifier = input.trim().to_lowercase();
            identifier = kebab_to_snake_case(&identifier);
            validate_name(&identifier, "index")?;

            let identifer_copy = identifier.clone();

            humanize_message(
                "\n Ok, creating a new index with the values you've set... ⚙️".to_string(),
            );
            create(InitCommand {
                name: Some(identifier),
                path: Some(std::path::PathBuf::from(".")),
                namespace,
                native: false,
                absolute_paths: true,
            })?;
            humanize_message(PROJECT_INITIALIZED.to_string());
            humanize_message("\n Here is the manifest file we created: \n\n".to_string());

            let manifest_name = format!("{}.manifest.yaml", identifer_copy);
            let manifest_path = format!("./{}", manifest_name);
            let manifest_content = fs::read_to_string(&manifest_path)?;

            for line in manifest_content.lines() {
                println!("{}", line.trim().bright_green());
                thread::sleep(time::Duration::from_millis(22));
            }

            humanize_message(DEPLOY_QUESTION.to_string());

            input = process_std(input);
            deploy_to_network(input, manifest_path)?;
        }
        _ => {
            println!("Invalid input. Please enter Y or n");
        }
    }
    Ok(())
}

fn render_greeter() {
    humanize_message("\n Welcome to the Fuel Indexer CLI 🚀".to_string());
    thread::sleep(time::Duration::from_millis(500));
    humanize_message(
        "\n This tool will help you understand how to create and deploy an index on the Fuel blockchain."
        .to_string()
    );
    thread::sleep(time::Duration::from_millis(500));
    humanize_message("\n Let's get started!".to_string());
    thread::sleep(time::Duration::from_millis(500));
    humanize_message("\n First, we'll create a new index.".to_string());
    thread::sleep(time::Duration::from_millis(500));
}

fn deploy_to_network(mut input: String, manifest: String) -> anyhow::Result<()> {
    match input.trim().to_lowercase().as_str() {
        "y" | "yes" => {
            humanize_message(
                "\n Connect to which network? 🤔 \n1. Local node\n2. Testnet \n > "
                    .to_string(),
            );

            input = process_std(input);
            match input.trim().to_lowercase().as_str() {
                "1" => {
                    start(init_start(Network::Local))?;
                    build(init_build(&manifest))?;
                    deploy(init_deploy(&manifest))?;
                }
                "2" => {
                    start(init_start(Network::Testnet))?;
                    build(init_build(&manifest))?;
                    deploy(init_deploy(&manifest))?;
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
    Ok(())
}

fn humanize_message(output: String) {
    for c in output.chars() {
        print!("{}", c.to_string().bright_yellow());
        io::stdout().flush().unwrap();
        let sleep_time = thread_rng().gen_range(20..92);
        thread::sleep(time::Duration::from_millis(sleep_time as u64));
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

fn init_build(manifest: &str) -> BuildCommand {
    humanize_message("\n Building indexer...".to_string());
    BuildCommand {
        manifest: Some(manifest.to_owned()),
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

fn init_deploy(manifest: &str) -> DeployCommand {
    humanize_message("\n Deploying indexer...".to_string());
    DeployCommand {
        url: "http://127.0.0.1:29987".to_string(),
        manifest: Some(manifest.to_owned()),
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
