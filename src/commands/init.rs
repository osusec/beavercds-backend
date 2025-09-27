use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;
use std::process::exit;
use tracing::error;

use crate::init::{self as init, templatize_init};
use crate::{access_handlers::frontend, commands::deploy};

pub fn run(interactive: &bool, blank: &bool) -> Result<()> {
    let options = if *interactive {
        init::interactive_init()?
    } else if *blank {
        init::blank_init()
    } else {
        init::example_init()
    };

    let configuration = templatize_init(options).context("could not render template")?;

    let mut f = File::create("rcds.yaml")?;
    f.write_all(configuration.as_bytes())?;

    // Note about external-dns
    println!("Note: external-dns configuration settings will need to be provided in rcds.yaml after file creation, under the `profiles.name.dns` key.");
    println!("Reference: https://github.com/bitnami/charts/tree/main/bitnami/external-dns");

    Ok(())
}
