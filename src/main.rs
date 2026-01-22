use anyhow::Result;
use clap::Parser;
use hsmm::cli::{run, Cli};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Check if --version or --help is requested (clap will handle these and exit)
    // Don't print banner for these special flags
    let args: Vec<String> = std::env::args().collect();
    let is_version_or_help = args.iter().any(|arg| arg == "--version" || arg == "-V" || arg == "--help" || arg == "-h");

    if !is_version_or_help {
        print_banner();
    }

    let cli = Cli::parse();

    // Initialize logging
    init_logging(cli.verbose, cli.log.as_ref())?;

    // Run the CLI command
    if let Err(e) = run(cli).await {
        tracing::error!("Error: {}", e);
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

fn init_logging(verbose: bool, log_file: Option<&std::path::PathBuf>) -> Result<()> {
    let env_filter = if verbose {
        "hsmm=debug,info"
    } else {
        "hsmm=info,warn,error"
    };

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_level(true);

    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| env_filter.into()),
        )
        .with(fmt_layer);

    // If log file is specified, also log to file
    if let Some(log_path) = log_file {
        // Create parent directory if it doesn't exist
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;

        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(std::sync::Arc::new(log_file))
            .with_ansi(false);

        subscriber.with(file_layer).init();
    } else {
        subscriber.init();
    }

    Ok(())
}

fn print_banner() {
    println!(
        r#"
▄▄
██
████▄ ▄█▀▀▀ ███▄███▄ ███▄███▄
██ ██ ▀███▄ ██ ██ ██ ██ ██ ██
██ ██ ▄▄▄█▀ ██ ██ ██ ██ ██ ██

Hytale Server Mod Manager
"#
    );
}
