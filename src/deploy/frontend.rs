use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Error, Ok, Result};
use itertools::Itertools;
use serde::Serialize;
use tracing::{debug, error, info, trace, warn};
use ureq;

use crate::builder::BuildResult;
use crate::configparser::challenge::{ExposeType, FlagType};
use crate::configparser::config::ProfileConfig;
use crate::configparser::{enabled_challenges, get_config, get_profile_config, ChallengeConfig};
use crate::utils::render_strict;

use super::kubernetes::KubeDeployResult;
use super::s3::S3DeployResult;

#[derive(Debug, Serialize)]
pub struct FrontendChalData {
    id: String,
    name: String,
    author: String,
    category: String,
    description: String,
    min_points: u32,
    max_points: u32,
    flag: String,
    files: Vec<String>,
}

/// Post collected challenge info structs to frontend. Returns the response
/// from frontend.
pub async fn update_frontend(
    profile_name: &str,
    chal_infos: &[FrontendChalData],
) -> Result<String> {
    let profile = get_profile_config(profile_name)?;

    let resp = ureq::post(format!("{}/chals/resolvestate", profile.frontend_url))
        .header("Authorization", format!("Token {}", profile.frontend_token))
        .send_json(chal_infos)
        .context("could not update frontend with challenge info")?;
    let body = resp.into_body().read_to_string()?;

    debug!("got response from frontend: {:?}", body);

    Ok(body)
}

/// Sync deployed challenges with rCTF frontend
pub async fn render_frontend_info(
    profile_name: &str,
    chal: &ChallengeConfig,
    build_result: &BuildResult,
    kube_result: &KubeDeployResult,
    s3_result: &S3DeployResult,
) -> Result<FrontendChalData> {
    let profile = get_profile_config(profile_name)?;
    let enabled_challenges = enabled_challenges(profile_name)?;

    // collect and render challenge info

    let hostname = chal_domain(chal, &profile.challenges_domain);
    let rendered_desc = render_strict(
        &chal.description,
        minijinja::context! {
            challenge => chal,
            host => hostname,
            hostname => hostname,
            port => chal_port(chal),
            nc => format!("`nc {} {}`", hostname, chal_port(chal)),
            url => format!("[https://{hostname}](https://{hostname})", ),
            link => format!("https://{hostname}"),
        },
    )?;

    let flag = match &chal.flag {
        FlagType::RawString(f) => f.clone(),
        FlagType::File { file } => {
            let full_path = chal.directory.join(file);
            let mut flag = String::new();
            let f = File::open(&full_path)
                .with_context(|| {
                    format!(
                        "could not open flag file {:?} for challenge {:?}",
                        &full_path, chal.directory
                    )
                })?
                .read_to_string(&mut flag);
            flag
        }
        FlagType::Text { text } => text.clone(),
        FlagType::Regex { regex } => unimplemented!("flag regex not implemented"),
        FlagType::Verifier { verifier } => unimplemented!("flag custom verifier not implemented"),
    };

    let chal_data = FrontendChalData {
        id: chal.slugify_slash(),
        name: chal.name.to_string(),
        author: chal.author.to_string(),
        category: chal.category.to_string(),
        description: rendered_desc,
        min_points: 0, // TODO! lookup
        max_points: 0, // TODO! lookup
        flag,
        files: s3_result.uploaded_asset_urls.clone(),
    };

    Ok(chal_data)
}

// TODO: move to impl ChallengeConfig?
// TODO: return Option and report errors when missing
fn chal_domain(chal: &ChallengeConfig, chal_domain: &str) -> String {
    // find first container with expose
    match chal.pods.iter().find(|p| !p.ports.is_empty()) {
        Some(p) => {
            let subdomain = match &p.ports[0].expose {
                ExposeType::Tcp(_port) => &chal.slugify_name(),
                ExposeType::Http(hostname) => hostname,
            };
            format!("{subdomain}.{chal_domain}")
        }
        // no pods have expose, no hostname for challenge
        None => "".to_string(),
    }
}

fn chal_port(chal: &ChallengeConfig) -> &i64 {
    // find first container with expose
    match chal.pods.iter().find(|p| !p.ports.is_empty()) {
        Some(p) => match &p.ports[0].expose {
            ExposeType::Tcp(port) => port,
            ExposeType::Http(_hostname) => &443,
        },
        // no pods have expose, no hostname for challenge
        None => &0,
    }
}
