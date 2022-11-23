use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use async_std::sync::Arc;
use clap::Parser;
use dashboard_example::helpers::generate_multi_wallet_config;
use fuels::{
    prelude::{CallParameters, Contract, Provider, TxParameters, WalletUnlocked},
    signers::Signer,
};
use fuels_abigen_macro::abigen;
use fuels_core::parameters::StorageConfiguration;
use fuels_signers::fuel_crypto::SecretKey;
use rand::Rng;
use std::{
    mem::size_of,
    path::{Path, PathBuf},
};
use tracing::info;
use tracing_subscriber::filter::EnvFilter;

abigen!(
    Dashboard,
    "examples/dashboard/contracts/out/debug/dashboard-abi.json"
);

#[derive(Debug, Parser, Clone)]
#[clap(name = "Indexer test web api", about = "Test")]
pub struct Args {
    #[clap(long, default_value = "127.0.0.1", help = "Test node host")]
    pub fuel_node_host: String,
    #[clap(long, default_value = "4000", help = "Test node port")]
    pub fuel_node_port: String,
    #[clap(long, help = "Test wallet filepath")]
    pub wallet_path: Option<PathBuf>,
    #[clap(long, help = "Contract bin filepath")]
    pub bin_path: Option<PathBuf>,
}

async fn preload_transfers(state: web::Data<Arc<AppState>>) -> impl Responder {
    let wallets = state.wallets.clone();
    let mut rng = rand::thread_rng();

    for wallet in wallets.iter() {
        let asset_id = rng.gen_range(0..10);
        let amount = rng.gen_range(0..10);
        for _ in 0..rng.gen_range(1..6) {
            let call_params = CallParameters::new(Some(1_000_000), None, None);
            let _ = state
                .contract
                .methods()
                .create_transfer(amount, [asset_id; 32].into(), wallet.address().into())
                .append_variable_outputs(1)
                .tx_params(TxParameters::default())
                .call_params(call_params)
                .call()
                .await;
        }
    }

    HttpResponse::Ok()
}

pub struct AppState {
    pub contract: Dashboard,
    pub wallets: Vec<WalletUnlocked>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| {
        let p = Path::new(file!())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap();

        p.display().to_string()
    });

    let manifest_dir = Path::new(&manifest_dir);

    let filter = match std::env::var_os("RUST_LOG") {
        Some(_) => {
            EnvFilter::try_from_default_env().expect("Invalid `RUST_LOG` provided")
        }
        None => EnvFilter::new("info"),
    };

    tracing_subscriber::fmt::Subscriber::builder()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .init();

    let wallets_config = generate_multi_wallet_config();

    let provider = Provider::connect("127.0.0.1:4000").await.unwrap();
    const SIZE_SECRET_KEY: usize = size_of::<SecretKey>();
    const PADDING_BYTES: usize = SIZE_SECRET_KEY - size_of::<u64>();
    let mut secret_key: [u8; SIZE_SECRET_KEY] = [0; SIZE_SECRET_KEY];

    let mut wallets: Vec<_> = (1..=wallets_config.num_wallets())
        .map(|wallet_counter| {
            secret_key[PADDING_BYTES..].copy_from_slice(&wallet_counter.to_be_bytes());

            WalletUnlocked::new_from_private_key(
                SecretKey::try_from(secret_key.as_slice())
                    .expect("This should never happen as we provide a [u8; SIZE_SECRET_KEY] array"),
                None,
            )
        })
        .collect();

    wallets
        .iter_mut()
        .for_each(|wallet| wallet.set_provider(provider.clone()));

    let opts = Args::from_args();
    let bin_path = opts.bin_path.unwrap_or_else(|| {
        Path::join(
            manifest_dir,
            "../../contracts/out/debug/fuel-indexer-test.bin",
        )
    });

    let bin_path_str = bin_path.as_os_str().to_str().unwrap();
    let _compiled = Contract::load_contract(bin_path_str, &None).unwrap();

    let contract_id = Contract::deploy(
        bin_path_str,
        &wallets[0],
        TxParameters::default(),
        StorageConfiguration::default(),
    )
    .await
    .unwrap();

    info!("Using contract at {}", contract_id.to_string());

    let contract = Dashboard::new(contract_id, wallets[0].clone());

    info!("Starting server at {}", "127.0.0.1:8000");

    let state = web::Data::new(Arc::new(AppState { contract, wallets }));

    let _ = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/preload_transfers", web::post().to(preload_transfers))
    })
    .bind("127.0.0.1:8000")
    .unwrap()
    .run()
    .await;

    Ok(())
}
