use anyhow::{bail, Context, Result};
use serde::Deserialize;
use tracing::debug;
use ureq::tls::{RootCerts, TlsConfig};
use ureq::Agent;

use crate::configparser::{get_config, get_profile_config};

#[derive(Deserialize, Debug)]
/// Returned JSON object from frontend (`{"status":"ok"}`)
struct CheckAccessResponse {
    status: String,
}

/// frontend dashboard access checks
pub fn check(profile_name: &str) -> Result<()> {
    let profile = get_profile_config(profile_name)?;

    debug!("checking frontend access at {}", profile.frontend_url);

    let agent = Agent::config_builder()
        .tls_config(
            TlsConfig::builder()
                .root_certs(RootCerts::PlatformVerifier)
                .build(),
        )
        .build()
        .new_agent();

    let resp = agent
        .get(format!("{}/api/checkaccess", profile.frontend_url))
        .header("Authorization", format!("Token {}", profile.frontend_token))
        .call()
        .context("could not reach frontend API")?;

    if resp.status() != 200 {
        bail!(
            "frontend returned unexpected status code: {}",
            resp.status()
        );
    }

    // also check content to make sure we're hitting frontend API
    let body: CheckAccessResponse = resp
        .into_body()
        .read_json()
        .context("frontend response was not valid JSON")?;

    if body.status != "ok" {
        bail!("frontend returned unexpected status: {:?}", body);
    }

    Ok(())
}
