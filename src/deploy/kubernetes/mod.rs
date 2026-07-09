use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Error, Ok, Result};
use base64ct::{Base64, Encoding};
use bollard::auth::DockerCredentials;
use itertools::Itertools;
use k8s_openapi::api::core::v1::Namespace;
use kube::api::DynamicObject;
use minijinja;
use tokio::time::timeout;
use tracing::{debug, error, info, trace, warn};

use crate::builder::BuildResult;
use crate::clients::{apply_manifest_yaml, kube_client, multidoc_deserialize, wait_for_status};
use crate::configparser::challenge::{ExposeType, Manifest, Pod, PodType};
use crate::configparser::config::ProfileConfig;
use crate::configparser::{get_config, get_profile_config, ChallengeConfig};
use crate::utils::{render_strict, TryJoinAll};

pub mod templates;

/// How and where a challenge was deployed/exposed at
pub struct KubeDeployResult {
    // challenges could have multiple exposed services
    pub exposed: Vec<PodDeployResult>,
}

pub enum PodDeployResult {
    Http { domain: String },
    Tcp { port: usize },
}

// Deploy all K8S resources for a single challenge `chal`.
//
// Creates the challenge namespace, deployments, services, and ingresses needed
// to deploy and expose the challenge.
pub async fn apply_challenge_resources(
    profile_name: &str,
    chal: &ChallengeConfig,
) -> Result<KubeDeployResult> {
    info!(
        "  deploying kube resources for chal {:?}...",
        chal.directory
    );
    // render templates

    let profile = get_profile_config(profile_name)?;

    let kube = kube_client(profile).await?;

    // Deploy our standard namespace regardless of whether challenge pods use
    // our template or a custom manifest. The custom manifest may set its own
    // namespace, but this lets us track challenge presence for both types.

    let namespace = format!("rcds-{}", chal.slugify());
    let ns_manifest = render_strict(
        templates::CHALLENGE_NAMESPACE,
        minijinja::context! { chal, namespace },
    )?;
    trace!("NAMESPACE:\n{}", ns_manifest);

    debug!("applying namespace for chal {:?}", chal.directory);

    // apply namespace manifest
    apply_manifest_yaml(&kube, &ns_manifest)
        .await?
        .iter()
        // and then wait for it to be ready
        .map(|object| wait_for_status(&kube, object))
        .try_join_all()
        .await?;

    // add cluster image pull secrets from config to new namespace
    deploy_pull_secrets(chal, profile_name, &namespace).await?;

    // namespace boilerplate over, deploy actual challenge pods

    let results = KubeDeployResult { exposed: vec![] };

    let _ = &chal
        .pods
        .iter()
        .map(|pod_type| async {
            match pod_type {
                PodType::Template(pod) => deploy_template_pod(chal, profile_name, pod)
                    .await
                    .with_context(|| {
                        format!(
                            "failed to deploy kube resources for challenge {:?} pod {:?}",
                            chal.slugify_slash(),
                            pod.name
                        )
                    }),
                PodType::Manifest(manifest) => deploy_custom_manifest(chal, profile_name, manifest)
                    .await
                    .with_context(|| {
                        format!(
                            "failed to deploy custom manifest {:?} for pod {:?}",
                            manifest.manifest_path, manifest.name
                        )
                    }),
            }
        })
        .try_join_all()
        .await?;

    Ok(results)
}

/// Deploy imagepullsecret from config into given namespace
async fn deploy_pull_secrets(
    chal: &ChallengeConfig,
    profile_name: &str,
    namespace: &str,
) -> Result<()> {
    let profile = get_profile_config(profile_name)?;
    let kube = kube_client(profile).await?;

    // add image pull credentials to the new namespace
    debug!(
        "applying namespace pull credentials for chal {:?}",
        chal.directory
    );

    let registry = &get_config()?.registry;
    let creds_manifest = render_strict(
        templates::IMAGE_PULL_CREDS_SECRET,
        minijinja::context! {
            namespace,
            slug => chal.slugify(),
            registry_domain => registry.domain,
            creds_b64 => Base64::encode_string(format!("{}:{}",
                registry.cluster.user,
                registry.cluster.pass,
            ).as_bytes()),
        },
    )?;
    apply_manifest_yaml(&kube, &creds_manifest).await?;

    Ok(())
}

/// Deploy challenge pod using our standard pod template
async fn deploy_template_pod(chal: &ChallengeConfig, profile_name: &str, pod: &Pod) -> Result<()> {
    let profile = get_profile_config(profile_name)?;
    let kube = kube_client(profile).await?;

    let pod_image = chal.container_tag_for_pod(profile_name, &pod.name)?;
    let depl_manifest = render_strict(
        templates::CHALLENGE_DEPLOYMENT,
        minijinja::context! {
            chal, pod, pod_image, profile_name,
            slug => chal.slugify(),
        },
    )?;
    trace!("DEPLOYMENT:\n{}", depl_manifest);

    debug!(
        "applying deployment for chal {:?} pod {:?}",
        chal.directory, pod.name
    );
    let depl = apply_manifest_yaml(&kube, &depl_manifest).await?;
    for object in depl {
        // wait for objects to be ready, with 5m timeout
        timeout(Duration::from_secs(5 * 60), wait_for_status(&kube, &object))
            .await
            // timeout wraps with another Result
            .with_context(|| {
                format!(
                    "timed out waiting for chal {:?} pod {:?} deployment to become ready",
                    chal.directory, pod.name
                )
            })?
            // inner result from wait_for_status
            .with_context(|| {
                format!(
                    "failed to get status for chal {:?} pod {:?} deployment",
                    chal.directory, pod.name
                )
            })?;
    }

    // tcp and http exposes need to be handled separately, so separate them by type
    let (tcp_ports, http_ports): (Vec<_>, Vec<_>) = pod
        .ports
        .iter()
        .partition(|p| matches!(p.expose, ExposeType::Tcp(_)));

    if !tcp_ports.is_empty() {
        let tcp_manifest = render_strict(
            templates::CHALLENGE_SERVICE_TCP,
            minijinja::context! {
                chal, pod, tcp_ports,
                slug => chal.slugify(), name_slug => chal.slugify_name(), domain => profile.challenges_domain
            },
        )?;
        trace!("TCP SERVICE:\n{}", tcp_manifest);

        debug!(
            "applying tcp service for chal {:?} pod {:?}",
            chal.directory, pod.name
        );
        let tcp = apply_manifest_yaml(&kube, &tcp_manifest).await?;
        for object in tcp {
            // wait for objects to be ready, with 5m timeout
            timeout(Duration::from_secs(5 * 60), wait_for_status(&kube, &object))
                .await
                // timeout wraps with another Result
                .with_context(|| {
                    format!(
                        "timed out waiting for chal {:?} pod {:?} exposed TCP service to become ready",
                        chal.directory, pod.name
                    )
                })?
                // inner result from wait_for_status
                .with_context(|| {
                    format!(
                        "failed to get status for chal {:?} pod {:?} exposed TCP service",
                        chal.directory, pod.name
                    )
                })?;
        }

        // TODO:
        // expose_results.exposed.push(PodDeployResult::Tcp { port: tcp_ports[0]. });
    }

    if !http_ports.is_empty() {
        let http_manifest = render_strict(
            templates::CHALLENGE_SERVICE_HTTP,
            minijinja::context! {
                chal, pod, http_ports,
                slug => chal.slugify(), domain => profile.challenges_domain
            },
        )?;
        trace!("HTTP INGRESS:\n{}", http_manifest);

        debug!(
            "applying http service and ingress for chal {:?} pod {:?}",
            chal.directory, pod.name
        );
        let ingress = apply_manifest_yaml(&kube, &http_manifest).await?;
        for object in ingress {
            // wait for objects to be ready, with 5m timeout
            timeout(Duration::from_secs(5 * 60), wait_for_status(&kube, &object))
                .await
                // timeout wraps with another Result
                .with_context(|| {
                    format!(
                        "timed out waiting for chal {:?} pod {:?} ingress to become ready",
                        chal.directory, pod.name
                    )
                })?
                // inner result from wait_for_status
                .with_context(|| {
                    format!(
                        "failed to get status for chal {:?} pod {:?} ingress",
                        chal.directory, pod.name
                    )
                })?;
        }
    }

    Ok(())
}

/// Deploy custom manifest resources for challenge pod
///
/// This applies all objects in the given manifest file AS-IS with no
/// modifications -- with the exception of adding the BCDS registry pull secrets
/// to any included namespaces.
async fn deploy_custom_manifest(
    chal: &ChallengeConfig,
    profile_name: &str,
    manifest: &Manifest,
) -> Result<()> {
    let profile = get_profile_config(profile_name)?;
    let kube = kube_client(profile).await?;

    // Normalize manifest path, as it is given as relative to the challenge
    // directory, not the top-level of challenge repo that this works at
    let full_path = chal.directory.join(&manifest.manifest_path);
    full_path
        .canonicalize()
        .with_context(|| format!("unable to read manifest at path {:?}", full_path))?;

    // Read in the given manifest
    let mut manifest_file = File::open(full_path)?;
    let mut manifest = String::new();
    manifest_file.read_to_string(&mut manifest)?;

    let applied_resources = apply_manifest_yaml(&kube, &manifest).await?;

    // Find any namespaces that were included in the manifest
    let namespaces = applied_resources
        .iter()
        .filter(|obj| obj.types.clone().unwrap_or_default().kind == "Namespace")
        .collect_vec();

    for obj in namespaces {
        let ns = obj
            .metadata
            .name
            .as_ref()
            .ok_or(anyhow!("Namespace object missing name field?"))?;
        deploy_pull_secrets(chal, profile_name, ns)
            .await
            .with_context(|| format!("unable to deploy image pull secrets into discovered namespace {:?} from manifest", ns))?;
    }

    Ok(())
}

// Updates the current ingress controller chart with the current set of TCP
// ports needed for challenges.
// TODO: move to Gateway to avoid needing to redeploy ingress?
// TODO: is this needed? currently TCP challenges are separate LoadBalancer svcs
// async fn update_ingress_tcp() -> Result<()> {
//     Ok(())
// }
