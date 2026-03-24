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

    let config_yaml = init::render_config_file(*interactive, *placeholders, *blank)?;

    let mut f = File::create("rcds.yaml")?;
    f.write_all(config_yaml.as_bytes())?;

    Ok(())
}
