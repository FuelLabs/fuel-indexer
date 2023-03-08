use crate::cli::AuthCommand;
use reqwest::{blocking::Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::{error, info};

#[derive(Deserialize)]
struct NonceResponse {
    nonce: String,
}

#[derive(Deserialize)]
struct SignatureResponse {
    token: Option<String>,
}

#[derive(Serialize)]
struct SignatureRequest {
    signature: String,
}

fn derive_signature_from_output(o: &str) -> String {
    o.split(':').last().unwrap().trim().to_string()
}

pub fn init(command: AuthCommand) -> anyhow::Result<()> {
    let AuthCommand { url, account, .. } = command;

    let target = format!("{url}/api/auth/nonce");

    let res = Client::new()
        .get(&target)
        .send()
        .expect("Failed to deploy indexer.");

    if res.status() != StatusCode::OK {
        error!(
            "\n❌ {} returned a non-200 response code: {:?}",
            &target,
            res.status()
        );
        return Ok(());
    }

    let response: NonceResponse = res.json().unwrap();

    let signature = match Command::new("forc-wallet")
        .arg("sign")
        .arg("--account")
        .arg(&account)
        .arg("string")
        .arg(&response.nonce)
        .output()
    {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let msg = stdout
                .strip_suffix('\n')
                .expect("Failed to capture signature output.");
            derive_signature_from_output(&msg)
        }

        Err(e) => {
            anyhow::bail!("❌ Failed to sign nonce: {e}");
        }
    };

    let target = format!("{url}/api/auth/signature");

    let res = Client::new()
        .post(&target)
        .json(&SignatureRequest { signature })
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

    if response.token.is_some() {
        info!("\n✅ Successfully authenticated at {target}",);
    } else {
        error!("\n❌ Failed to produce a token.",);
    }

    Ok(())
}
