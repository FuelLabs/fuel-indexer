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
use rand::Rng;
use std::fs;
use std::{
    io::{self, Write},
    process::Command,
    thread, time,
};

const TITLE: &str = "

███████╗██╗░░░██╗███████╗██╗░░░░░  ██╗███╗░░██╗██████╗░███████╗██╗░░██╗███████╗██████╗░
██╔════╝██║░░░██║██╔════╝██║░░░░░  ██║████╗░██║██╔══██╗██╔════╝╚██╗██╔╝██╔════╝██╔══██╗
█████╗░░██║░░░██║█████╗░░██║░░░░░  ██║██╔██╗██║██║░░██║█████╗░░░╚███╔╝░█████╗░░██████╔╝
██╔══╝░░██║░░░██║██╔══╝░░██║░░░░░  ██║██║╚████║██║░░██║██╔══╝░░░██╔██╗░██╔══╝░░██╔══██╗
██║░░░░░╚██████╔╝███████╗███████╗  ██║██║░╚███║██████╔╝███████╗██╔╝╚██╗███████╗██║░░██║
╚═╝░░░░░░╚═════╝░╚══════╝╚══════╝  ╚═╝╚═╝░░╚══╝╚═════╝░╚══════╝╚═╝░░╚═╝╚══════╝╚═╝░░╚═╝
";

const WELCOME_MANIFEST_PATH: &str = "welcome.manifest.yaml";
const PROJECT_INITIALIZED: &str =
    "\n Indexer project initialized. Manifest file created. ✅";
const DEPLOY_QUESTION: &str = "\n Start the indexer and deploy the index? (Y/n) \n > ";

pub async fn init(command: WelcomeCommand) -> anyhow::Result<()> {
    TITLE.lines().for_each(|line| {
        println!("{}", line.trim().bright_cyan());
        thread::sleep(time::Duration::from_millis(50));
    });

    let WelcomeCommand { greeter, verbose } = command;
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
                verbose: true,
            })?;
            humanize_message(PROJECT_INITIALIZED.to_string());
            if verbose {
                let output = Command::new("tree")
                    .output()
                    .expect("failed to execute tree command");

                if output.status.success() {
                    let tree = String::from_utf8(output.stdout).unwrap();
                    humanize_message(tree);
                } else {
                    let error = String::from_utf8(output.stderr).unwrap();
                    humanize_message(error);
                }
            }
            humanize_message(DEPLOY_QUESTION.to_string());

            input = process_std(input);
            deploy_to_network(input, WELCOME_MANIFEST_PATH.to_string()).await?;
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
                verbose: true,
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
            deploy_to_network(input, manifest_path).await?;
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

async fn deploy_to_network(mut input: String, manifest: String) -> anyhow::Result<()> {
    match input.trim().to_lowercase().as_str() {
        "y" | "yes" => {
            humanize_message(
                "\n Connect to which network? 🤔 \n1. Local node\n2. Testnet \n > "
                    .to_string(),
            );

            let build_command = BuildCommand {
                manifest: Some(manifest.to_owned()),
                ..Default::default()
            };
            let deploy_command = DeployCommand {
                manifest: Some(manifest.to_owned()),
                ..Default::default()
            };

            input = process_std(input);
            match input.trim().to_lowercase().as_str() {
                "1" => {
                    //init for local node
                    let start_command = StartCommand {
                        fuel_node_host: "http://127.0.0.1:29987".to_string(),
                        fuel_node_port: "29987".to_string(),
                        graphql_api_host: defaults::GRAPHQL_API_HOST.to_string(),
                        graphql_api_port: defaults::GRAPHQL_API_PORT.to_string(),
                        ..Default::default()
                    };
                    start(start_command).await?;
                    build(build_command)?;
                    deploy(deploy_command)?;
                }
                "2" => {
                    //init for testnet
                    let start_command = StartCommand {
                        fuel_node_host: "https://node-beta-2.fuel.network/graphql"
                            .to_string(),
                        fuel_node_port: "4000".to_string(),
                        graphql_api_host: "node-beta-2.fuel.network".to_string(),
                        graphql_api_port: "80".to_string(),
                        ..Default::default()
                    };
                    start(start_command).await?;
                    build(build_command)?;
                    deploy(deploy_command)?;
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
    output.chars().for_each(|c| {
        print!("{}", c.to_string().bright_yellow());
        io::stdout().flush().expect("failed to flush stdout");
        let sleep_time = rand::thread_rng().gen_range(20..92);
        thread::sleep(time::Duration::from_millis(sleep_time));
    })
}

fn process_std(mut input: String) -> String {
    input.clear();
    io::stdout().flush().expect("failed to flush stdout");
    io::stdin()
        .read_line(&mut input)
        .expect("failed to read from stdin");
    input
}
