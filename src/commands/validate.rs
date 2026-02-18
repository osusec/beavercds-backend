use anyhow::{bail, Result};
use itertools::Itertools;
use std::path::Path;
use std::process::exit;
use tracing::{debug, error, info, trace, warn};

use crate::configparser::{get_challenges, get_config, get_profile_deploy};

pub fn run() -> Result<()> {
    info!("validating config...");

    let config = get_config()?;

    // is point class max/min order correct?
    for class in config.point_classes.iter() {
        if class.min > class.max {
            bail!(
                "min/max points are backwards for point class '{}' (min: {} > max: {})",
                class.name,
                class.min,
                class.max
            )
        }
    }

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

    // are all challenge ids unique?
    // find any challenges with duplicate ids
    let dups = chals
        .iter()
        .duplicates_by(|c| &c.challenge_id)
        .collect_vec();
    if !dups.is_empty() {
        // fetch the other challenge with the duplicate id. duplicates() only
        // returns the second duplicating item, not both, so need to get it.
        let duped_chals = chals
            .iter()
            .filter(|c| dups.iter().any(|d| d.challenge_id == c.challenge_id))
            .map(|c| c.slugify_slash())
            .collect_vec();
        bail!("challenge IDs for chals {:?} conflict", duped_chals);
    }

    info!("  challenges ok!");

    // check global deploy settings for invalid challenges
    info!("validating deploy config...");
    for (profile_name, _pconfig) in config.profiles.iter() {
        // fetch from config
        let deploy_challenges = get_profile_deploy(profile_name)?;

        // check for missing
        let missing: Vec<_> = deploy_challenges
            .challenges
            .keys()
            .filter(
                // try to find any challenge paths in deploy config that do not exist
                |path| !chals.iter().any(|c| c.directory == Path::new(path)),
            )
            .collect();

        // TODO: figure out how to return this error directly
        if !missing.is_empty() {
            error!(
                "deploy settings for profile '{profile_name}' has challenges that do not exist:"
            );
            missing.iter().for_each(|path| error!("  - {path}"));
            bail!("failed to validate deploy config");
        }
    }
    info!("  deploy ok!");

    Ok(())
}
