use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Error, Ok, Result};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, trace, warn};
use ureq::tls::{RootCerts, TlsConfig};
use ureq::Agent;

use crate::builder::BuildResult;
use crate::configparser::challenge::{ExposeType, FlagType, PodType};
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

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // used to validate response schema only
pub struct FrontendResolveResponse {
    current: Vec<String>,
    removed: Vec<String>,
}

/// Post collected challenge info structs to frontend. Returns the response
/// from frontend.
pub async fn update_frontend(
    profile_name: &str,
    chal_infos: &[FrontendChalData],
) -> Result<FrontendResolveResponse> {
    let profile = get_profile_config(profile_name)?;
    let config = get_config()?;

    info!("updating frontend with challenge info...");

    let agent = Agent::config_builder()
        .tls_config(
            TlsConfig::builder()
                .root_certs(RootCerts::PlatformVerifier)
                .build(),
        )
        .build()
        .new_agent();

    // POST collected challenge data to frontend.
    let resp = agent
        .post(format!("{}/api/resolvestate", profile.frontend_url))
        .header("Authorization", format!("Token {}", profile.frontend_token))
        .send_json(chal_infos)
        .context("could not update frontend with challenge info")?;

    let body: FrontendResolveResponse = resp
        .into_body()
        .read_json()
        .context("got malformed response from frontend")?;

    debug!("got response from frontend: {:?}", body);

    // POST bracket config to that endpoint as well.
    let resp = agent
        .post(format!("{}/api/updatebrackets", profile.frontend_url))
        .header("Authorization", format!("Token {}", profile.frontend_token))
        .send_json(&config.brackets)
        .context("could not update frontend with brackets config")?;
    // Frontend uses the same current/removed response schema as the chals
    // update.
    let body: FrontendResolveResponse = resp
        .into_body()
        .read_json()
        .context("got malformed response from frontend")?;

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
    let config = get_config()?;
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
                        full_path, chal.directory
                    )
                })?
                .read_to_string(&mut flag);
            flag
        }
        FlagType::Text { text } => text.clone(),
        FlagType::Regex { regex } => unimplemented!("flag regex not implemented"),
        FlagType::Verifier { verifier } => unimplemented!("flag custom verifier not implemented"),
    };

    // Get point class from challenge or use default from repo config
    let point_class_name = chal
        .point_class
        .as_ref()
        .unwrap_or(&config.defaults.point_class);
    // Then look the class name up in the repo config
    let point_info = config
        .point_classes
        .iter()
        .find(|class| class.name == *point_class_name)
        .ok_or(anyhow!("challenge points are missing in config"))?;

    let chal_data = FrontendChalData {
        id: chal.challenge_id.to_string(),
        name: chal.name.to_string(),
        author: chal.author.to_string(),
        category: chal.category.to_string(),
        description: rendered_desc,
        min_points: point_info.min,
        max_points: point_info.max,
        flag,
        files: s3_result.uploaded_asset_urls.clone(),
    };

    Ok(chal_data)
}

// TODO: move to impl ChallengeConfig?
// TODO: return Option and report errors when missing
fn chal_domain(chal: &ChallengeConfig, chal_domain: &str) -> String {
    // find first container with expose
    let first_expose = chal
        .pods
        .iter()
        // find first non-custom-manifest pod
        .filter_map(|pod_type| match &pod_type {
            PodType::Template(template) => Some(&template.ports),
            PodType::Manifest(_) => None,
        })
        // with expose config
        .find_map(|ports| ports.first());

    match first_expose {
        Some(pc) => {
            let subdomain = match &pc.expose {
                // TODO: drop the specific port and go with static port + chosen
                // hostname? ask for both hostname/port? this is hacky to
                // automatically generate a name here
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
    let first_expose = chal
        .pods
        .iter()
        // find first non-custom-manifest pod
        .filter_map(|pod_type| match &pod_type {
            PodType::Template(template) => Some(&template.ports),
            PodType::Manifest(_) => None,
        })
        // with expose config
        .find_map(|ports| ports.first());

    match first_expose {
        Some(pc) => match &pc.expose {
            ExposeType::Tcp(port) => port,
            ExposeType::Http(_hostname) => &443,
        },
        // no pods have expose, no hostname for challenge
        None => &0,
    }
}
