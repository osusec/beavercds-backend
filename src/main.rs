use beavercds_ng::commands;
use clap::Parser;
use simplelog::*;

mod cli;

fn main() {
    let cli = cli::Cli::parse();

    let log_config = ConfigBuilder::new()
        .set_time_level(LevelFilter::Off)
        .build();

    TermLogger::init(
        cli.verbose.log_level_filter(),
        log_config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();

    trace!("args: {:?}", cli);

    // dispatch commands
    match &cli.command {
        cli::Commands::Validate => commands::validate::run(),

        cli::Commands::CheckAccess {
            profile,
            kubernetes,
            frontend,
            registry,
        } => {
            commands::validate::run();
            commands::check_access::run(profile, kubernetes, frontend, registry)
        }

        #[allow(unused_variables)]
        cli::Commands::Build {
            profile,
            push,
            no_push,
        } => {
            commands::validate::run();
            commands::build::run(profile, &!no_push)
        }

        cli::Commands::Deploy {
            profile,
            no_build,
            dry_run,
        } => {
            commands::validate::run();
            commands::deploy::run(profile, no_build, dry_run)
        }

        cli::Commands::Init {
            interactive,
            blank
        } => commands::init::run(interactive, blank)
    }
}
