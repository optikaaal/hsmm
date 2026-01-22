use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "hsmm-web")]
#[command(version, about = "Hytale Server Mod Manager - Web UI", long_about = None)]
struct Args {
    /// Path to the configuration file
    #[arg(short, long, default_value = "mods.toml")]
    config: PathBuf,

    /// Mods directory
    #[arg(short = 'o', long, default_value = "mods")]
    output: PathBuf,

    /// Server files directory (for config.json, permissions.json, etc.)
    #[arg(short, long, default_value = ".")]
    server_files: PathBuf,

    /// Static files directory (for frontend dist)
    #[arg(long, default_value = "dist/web")]
    static_dir: PathBuf,

    /// Port to run the web server on
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logging with file output
    let log_level = if args.verbose {
        "hsmm=debug,hsmm_web=debug,axum=debug,tower_http=debug,access_log=debug,error_log=debug"
    } else {
        "hsmm=info,hsmm_web=info,axum=info,access_log=info,error_log=info"
    };

    // Create logs directory if it doesn't exist
    let logs_dir = args.server_files.join("logs");
    std::fs::create_dir_all(&logs_dir)?;

    // Configure file logging for web UI
    let log_file_path = logs_dir.join("web-ui.log");
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)?;

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| log_level.into()),
        )
        .with_writer(std::sync::Arc::new(log_file))
        .with_ansi(false) // Disable ANSI colors for file output
        .init();

    tracing::info!("Starting Hytale Server Mod Manager Web UI");
    tracing::info!("Logging to: {}", log_file_path.display());
    tracing::info!("Config: {}", args.config.display());
    tracing::info!("Mods directory: {}", args.output.display());
    tracing::info!("Server files: {}", args.server_files.display());

    // Create web server
    let server =
        hsmm::web::WebServer::new(args.config, args.output, args.server_files, args.static_dir);

    // Run server
    server.run(args.port).await?;

    Ok(())
}
