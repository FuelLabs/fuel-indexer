use crate::ops::{
    forc_index_build::init as build, forc_index_deploy::init as deploy,
    forc_index_init::create_indexer as create, forc_index_start::init as start,
};
use crate::{
    cli::{BuildCommand, DeployCommand, InitCommand, StartCommand, WelcomeCommand},
    utils::defaults,
};
use std::io::{self, Write};
use tracing::info;

enum Network {
    Local,
    Testnet,
}

const FUEL_LOGO: &str = r#"
                         .7J!:                    
                        ~5PP@@#J                  
                      :YPP5#@@@G                  
                     7PPPPP@@@&~                  
                   ~5PPPP5#@@@5                   
                 :YPPPPPPP@@@&^                   
                7PPPPPPP5&@@@Y                    
              ~5PPPPPPPPG@@@#:                    
            :YPPPPPPPPPPPGGGPJJJJJJJJJJJJ?~.      
           7PPPPPPPPPPPPPPPPPPPPPPPPPPPPB@@@G^    
         ~5PPPPPPPPPPPPPPPPPPPPPPPPPP5P&@@@@P:    
       :YPPPPPPPPPPPPPPPPPPPPPPPPPP5P#@@@@#~      
      7PPPPPPPPPPPPPPPPPPPPPPPPPPP5G@@@@&J        
    ^PPP5555555555555PPPPPPPPPPP5P&@@@@P:         
    .JG&&&&&&&&&&&&&&BPPPPPPPP5P#@@@@#~           
      .Y&@@@@@@@@@@@@GPPPPPPP5G@@@@&J             
        .~77777777775PPPPPP5P&@@@@P:              
                    YPPPP5P#@@@@#~                
                   !PPPP5G@@@@&J.                 
                  .PPP5P&@@@@P:                   
                  7P5P#@@@@#~                     
                 .PPB@@@@&J.                      
                  ~5&@@@P:                        
                    :YP!                          
"#;

const WELCOME_MANIFEST: &str = "welcome.manifest.yaml";
const WASM_TARGET: &str = "wasm32-unknown-unknown";

pub async fn init(command: WelcomeCommand) -> anyhow::Result<()> {
    info!("Create default index? (Y/n)");

    let mut input = String::new();
    input = process_std(input);

    match input.trim().to_lowercase().as_str() {
        "y" | "yes" => {
            info!("🏗️ Creating a default index...");
            create(InitCommand {
                name: Some("welcome".to_string()),
                path: Some(std::path::PathBuf::from(".")),
                namespace: "default".to_string(),
                native: false,
                absolute_paths: true,
            })?;

            info!("\n✅ Indexer project initialized. Manifest file created.");
            info!("\n Start the an indexer and deploy the index? (Y/n)");

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

            match input.trim().to_lowercase().as_str() {
                "1" => {
                    println!("Deploying indexer to local node...");
                }
                "2" => {
                    println!("Deploying indexer to testnet...");
                }
                _ => {
                    println!("Invalid input. Please enter 1 or 2");
                }
            }
        }
        "n" | "no" => {
            println!("Skipping default indexer creation...");
        }
        _ => {
            println!("Invalid input. Please enter Y or n");
        }
    }
    Ok(())
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
    println!("Starting indexer...");
    let mut start_command = StartCommand {
        log_level: "info".to_string(),
        config: None,
        manifest: Some(std::path::PathBuf::from(".")),
        fuel_node_host: String::new(),
        fuel_node_port: String::new(),
        graphql_api_host: defaults::GRAPHQL_API_HOST.to_string(),
        graphql_api_port: defaults::GRAPHQL_API_PORT.to_string(),
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
        }
        Network::Testnet => {
            start_command.fuel_node_host = "https://node-beta-2.fuel.network".to_string();
            start_command.fuel_node_port = "4000".to_string();
        }
    }
    start_command
}

fn init_build() -> BuildCommand {
    println!("Building indexer for local node deployment...");
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
    println!("Deploying indexer to local node...");
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
