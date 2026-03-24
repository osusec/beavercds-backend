use anyhow::{bail, Context, Result};
use inquire;
use owo_colors::OwoColorize;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::exit;
use tracing::{debug, error, info, warn};

use crate::init;

/// Initialize a blank directory into a beavercds challenge repo.
///
/// This will create a rcds.yaml, challenge creation script, and basic README
/// in the folder, optionally prompting the user to fill in info for the rcds
/// config file.
pub fn run(interactive: &bool, placeholders: &bool, blank: &bool, force: &bool) -> Result<()> {
    let config_yaml = init::render_config_file(*interactive, *placeholders, *blank)?;
    write_or_ask("rcds.yaml", &config_yaml, *force)?;

    write_or_ask(
        "./scripts/new-chal.py",
        init::templates::SCRIPTS_NEW_CHAL,
        *force,
    )?;
    write_or_ask("./README.md", init::templates::README, *force)?;

    Ok(())
}

// Write out `content` to `path`, but ask first if the file already exists.
// Returns Ok(true) if file was written, Ok(false) if user skipped overwriting,
// or Err if prompt or writing file failed.
fn write_or_ask(path: &str, content: &str, force: bool) -> Result<bool> {
    let path = Path::new(path);

    // Check if the file exists!
    match path.try_exists() {
        Ok(false) => (), // no file at that path, OK to continue
        Ok(true) => {
            // file exists, ask user
            if force {
                // go ahead anyway
                warn!("File already exists at {path:?}, but overwriting anyway due to --force")
            } else {
                match inquire::Confirm::new("File {path:?} already exists! Overwrite?")
                    .with_default(false)
                    .prompt()
                {
                    Ok(true) => (), // ok to overwrite
                    Ok(false) => {
                        // skip writing this file but continue to next
                        info!("Ok, not overwriting");
                        return Ok(false);
                    }
                    Err(err) => bail!("Error prompting user"),
                }
            }
        }
        Err(_) => bail!("Could not read info about file {path:?}"),
    }

    // create directory if needed and then write out
    if let Some(p) = path.parent() {
        if !p.try_exists().unwrap_or(true) {
            fs::create_dir(p)?;
        }
    }
    File::create(path)?.write_all(content.as_bytes())?;

    Ok(true)
}
