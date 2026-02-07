use anyhow::{bail, Context, Result};
use inquire;
use owo_colors::OwoColorize;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::exit;
use tracing::{debug, error, warn};

use crate::init;
use crate::{access_handlers::frontend, commands::deploy};

pub fn run(_interactive: &bool, placeholders: &bool, blank: &bool, force: &bool) -> Result<()> {
    // Check if the file exists!
    if let Ok(true) = Path::new("rcds.yaml").try_exists() {
        if *force {
            debug!("rcds.yaml config file exists but forcing overwrite");
        } else {
            match inquire::Confirm::new("An rcds.yaml config already exists! Overwrite?")
                .with_default(false)
                .prompt()
            {
                Ok(true) => (), // ok to overwrite
                Ok(false) => bail!("Not overwriting"),
                Err(err) => bail!("Error prompting user"),
            }
        }
    }

    let options = if *blank {
        init::blank_init()
    } else if *placeholders {
        init::placeholder_init()
    } else {
        // default to interactive if no flags given
        init::interactive_init()?
    };

    let configuration = init::templatize_init(&options).context("could not render template")?;

    let mut f = File::create("rcds.yaml")?;
    f.write_all(configuration.as_bytes())?;

    // Note about external-dns
    warn!("Note: external-dns configuration settings will need to be provided in rcds.yaml after file creation, under the `profiles.<name>.dns` key.");
    warn!("Reference: https://kubernetes-sigs.github.io/external-dns/latest/charts/external-dns/");

    Ok(())
}
