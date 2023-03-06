use crate::cli::{BuildCommand, InitCommand, WelcomeCommand, DeployCommand};
use crate::ops::{
    forc_index_build::init as build, forc_index_deploy::init as deploy,
    forc_index_init::create_indexer,
};
use std::io::{self, Write};

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

const WELCOME_MANIFEST: &str= "welcome.manifest.yaml";
const WASM_TARGET: &str= "wasm32-unknown-unknown";

pub async fn init(command: WelcomeCommand) -> anyhow::Result<()> {
    //start the fuel-indexer with fuel-node
    println!("Create default index? (Y/n)");

    let mut input = String::new();
    io::stdout().flush().expect("failed to flush stdout");
    io::stdin()
        .read_line(&mut input)
        .expect("failed to read from stdin");

    match input.trim().to_lowercase().as_str() {
        "y" | "yes" => {
            println!("Creating a default indexer...");
            create_indexer(InitCommand {
                name: Some("welcome".to_string()),
                path: Some(std::path::PathBuf::from(".")),
                namespace: "default".to_string(),
                native: false,
                absolute_paths: true,
            })?;

            let manifest_path= std::path::PathBuf::from("./welcome.manifest.yaml");
            let manifest_str = std::fs::read_to_string(&manifest_path)?;
            println!("Indexer project initialized. The manifest file has been created:");
            println!("{}", manifest_str);

            println!("Deploy the index? (Y/n)");

            input.clear();
            io::stdout().flush().expect("failed to flush stdout");
            io::stdin()
                .read_line(&mut input)
                .expect("failed to read from stdin");

            match input.trim().to_lowercase().as_str() {
                "y" | "yes" => {
                    println!("Connect to which network? \n1. Local node\n2. Testnet");

                    input.clear();
                    io::stdout().flush().expect("failed to flush stdout");
                    io::stdin()
                        .read_line(&mut input)
                        .expect("failed to read from stdin");

                    match input.trim().to_lowercase().as_str() {
                        "1" => {
                            println!("Starting a local fuel node...");

                            println!("Building indexer for local node deployment...");
                            build(init_build())?;
                            println!("Deploying indexer to local node...");
                            deploy(init_deploy(Network::Local))?;

                        }
                        "2" => {
                            println!("Deploying indexer to testnet...");
                            build(init_build())?;
                            println!("Deploying indexer to testnet...");
                            deploy(init_deploy(Network::Testnet))?;
                        }
                        _ => {
                            println!("Invalid input. Please enter 1 or 2");
                        }
                    }
                }
                "n" | "no" => {
                    println!("Skipping indexer deployment...");
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

fn init_build() -> BuildCommand {
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

fn init_deploy(network: Network) -> DeployCommand {
    let mut deploy_command = DeployCommand {
                url: String::new(),
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
    };
    match network {
        Network::Local => {
            deploy_command.url = "http://127.0.0.1:29987".to_string();
        }
        Network::Testnet => {
            deploy_command.url = "https://node-beta-2.fuel.network".to_string();
        }
    }

    deploy_command
}

