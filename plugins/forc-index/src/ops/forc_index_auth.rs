use crate::cli::AuthCommand;
use reqwest::{blocking::Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::{error, info};

#[derive(Deserialize, Debug)]
struct NonceResponse {
    uid: String,
    #[allow(unused)]
    expiry: u64,
}

#[derive(Deserialize, Debug)]
struct SignatureResponse {
    token: Option<String>,
}

#[derive(Serialize, Debug)]
struct SignatureRequest {
    signature: String,
    message: String,
}

fn derive_signature_from_output(o: &str) -> String {
    o.split(':').last().unwrap().trim().to_string()
}

pub fn init(command: AuthCommand) -> anyhow::Result<()> {
    let AuthCommand {
        url,
        account,
        verbose,
    } = command;

    let target = format!("{url}/api/auth/nonce");

    let res = Client::new()
        .get(&target)
        .send()
        .expect("Failed to deploy indexer.");

    if res.status() != StatusCode::OK {
        if verbose {
            error!(
                "\n❌ {} returned a non-200 response code: {:?}",
                &target,
                res.status()
            );
        } else {
            error!("\n❌ Action failed (Status({}))", res.status());
        }
        return Ok(());
    }

    let response: NonceResponse = res.json().unwrap();

    // NOTE: Until latest forc-wallet is available via fuelup, manually insert
    // the path to the latest compiled forc-wallet binary
    let signature = match Command::new("forc-wallet")
        .arg("sign")
        .arg("--account")
        .arg(&account)
        .arg("string")
        .arg(&response.uid)
        .output()
    {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let msg = stdout
                .strip_suffix('\n')
                .expect("Failed to capture signature output.");
            derive_signature_from_output(msg)
        }

        Err(e) => {
            anyhow::bail!("❌ Failed to sign nonce: {e}");
        }
    };

    let target = format!("{url}/api/auth/signature");

    let body = SignatureRequest {
        signature,
        message: response.uid,
    };

    let res = Client::new()
        .post(&target)
        .json(&body)
        .send()
        .expect("Failed post signature.");

    if res.status() != StatusCode::OK {
        error!(
            "\n❌ {} returned a non-200 response code: {:?}",
            &target,
            res.status()
        );
        return Ok(());
    }

    let response: SignatureResponse = res.json().unwrap();

    if let Some(token) = response.token {
        if verbose {
            info!(
                "\n✅ Successfully authenticated at {target}.\n\nToken: {}",
                token
            );
        } else {
            info!("\n✅ Authenticated successfully.\n\nToken: {}", token);
        }
    } else {
        error!("\n❌ Failed to produce a token.");
    }

    Ok(())
}
