use anyhow::{bail, Result};
use itertools::Itertools;
use std::path::Path;
use std::process::exit;
use tracing::{debug, error, info, trace, warn};

use crate::configparser::{get_challenges, get_config, get_profile_deploy};

pub fn run() -> Result<()> {
    info!("validating config...");

    let config = get_config()?;
    info!("  config ok!");

    info!("validating challenges...");
    // print these errors here instead of returning, since its a vec of them
    // TODO: figure out how to return this error directly
    let chals = match get_challenges() {
        Ok(c) => c,
        Err(errors) => {
            for e in errors.iter() {
                error!("{e:#}");
            }
            bail!("failed to validate challenges");
        }
    };
    // double check specific things about challenges
    for chal in chals {
        // does point class exist in default config?
        if let Some(class) = &chal.point_class {
            if !config.point_classes.iter().any(|p| &p.name == class) {
                bail!(
                    "point class '{}' for challenge {} does not exist in config",
                    chal.slugify_slash(),
                    class
                )
            }
        }
    }

    info!("  challenges ok!");

    // check global deploy settings for invalid challenges
    info!("validating deploy config...");
    for (profile_name, _pconfig) in config.profiles.iter() {
        // fetch from config
        let deploy_challenges = &get_profile_deploy(profile_name)?.challenges;
        let chal_slugs = chals.iter().map(|c| c.slugify_slash()).collect_vec();

        // check for chals defined in deploy: but don't exist in repo
        let missing = deploy_challenges
            .keys()
            .filter(
                // try to find any challenge paths in deploy config that do not exist
                |path| !chals.iter().any(|c| c.directory == Path::new(path)),
            )
            .collect_vec();

        // TODO: figure out how to return this error directly
        if !missing.is_empty() {
            error!(
                "deploy settings for profile '{profile_name}' has challenges that do not exist:"
            );
            missing.iter().for_each(|path| error!("  - {path}"));
            bail!("failed to validate deploy config");
        }

        // check for challenges found but not mentioned in deploy config
        let extra = chal_slugs
            .iter()
            .filter(|c| !deploy_challenges.contains_key(c.to_owned()))
            .collect_vec();

        if !extra.is_empty() {
            warn!("deploy settings for profile '{profile_name}' is missing challenges:");
            extra.iter().for_each(|path| warn!("  - {path}"));
        }
    }
    info!("  deploy ok!");

    Ok(())
}
