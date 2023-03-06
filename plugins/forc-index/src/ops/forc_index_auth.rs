use crate::{
    cli::AuthCommand,
    utils::{extract_manifest_fields, project_dir_info},
};
use reqwest::{
    blocking::{multipart::Form, Client},
    header::{HeaderMap, AUTHORIZATION},
    StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::{to_string_pretty, value::Value, Map};
use std::{
    collections::HashMap,
    fs,
    io::{BufReader, Read},
    process::Command,
};
use tracing::{error, info};

#[derive(Deserialize)]
struct NonceResponse {
    nonce: String,
}

#[derive(Deserialize)]
struct SignatureResponse {
    token: Option<String>,
    success: Option<String>,
    details: Option<String>,
}

#[derive(Serialize)]
struct SignatureRequest {
    signature: String,
}

fn derive_signature_from_output(o: &str) -> String {
    o.split(':').last().unwrap().trim().to_string()
}

pub fn init(command: AuthCommand) -> anyhow::Result<()> {
    let AuthCommand {
        url,
        manifest,
        path,
        verbose,
        account_index,
    } = command;

    let (_root_dir, manifest_path, _index_name) =
        project_dir_info(path.as_ref(), manifest.as_ref())?;

    let mut manifest_file = fs::File::open(&manifest_path)?;
    let mut manifest_contents = String::new();
    manifest_file.read_to_string(&mut manifest_contents)?;
    let manifest: serde_yaml::Value = serde_yaml::from_str(&manifest_contents)?;

    let (namespace, identifier, graphql_schema, module_path) =
        extract_manifest_fields(manifest, None)?;

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
        .arg("--account-index")
        .arg(&account_index)
        .arg("sign")
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
        .expect("Failed to deploy indexer.");

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
