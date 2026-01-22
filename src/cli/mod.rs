use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Hytale Server Mod Manager - Automatic mod updates from CurseForge
#[derive(Parser, Debug)]
#[command(name = "hsmm")]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Path to the configuration file
    #[arg(short, long, default_value = "mods.toml")]
    pub config: PathBuf,

    /// Mods directory
    #[arg(short, long, default_value = "mods")]
    pub output: PathBuf,

    /// Log file path (optional, logs to stdout if not specified)
    #[arg(short, long)]
    pub log: Option<PathBuf>,

    /// Verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add a mod to the configuration
    Add {
        /// Mod name
        name: String,

        /// CurseForge project ID or slug
        identifier: String,
    },

    /// Remove a mod from the configuration
    Remove {
        /// Mod name, ID, or slug
        identifier: String,
    },

    /// List all configured mods
    List,

    /// Update all mods to their latest versions
    Upgrade,

    /// Show configuration
    Config,
}

pub async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Add {
            ref name,
            ref identifier,
        } => add_mod(&cli, name.clone(), identifier.clone()).await,
        Commands::Remove { ref identifier } => remove_mod(&cli, identifier.clone()).await,
        Commands::List => list_mods(&cli).await,
        Commands::Upgrade => upgrade_mods(&cli).await,
        Commands::Config => show_config(&cli).await,
    }
}

async fn add_mod(cli: &Cli, name: String, identifier_str: String) -> Result<()> {
    let mut config = crate::config::read_config(&cli.config)?;

    // Parse identifier as either ID (number) or slug (string)
    let identifier = if let Ok(id) = identifier_str.parse::<i32>() {
        crate::config::ModIdentifier::project_id(id)
    } else {
        crate::config::ModIdentifier::project_slug(identifier_str)
    };

    crate::config::add_mod(&mut config, name.clone(), identifier)?;
    crate::config::write_config(&cli.config, &config)?;

    println!("✓ Added mod '{}' to configuration", name);
    Ok(())
}

async fn remove_mod(cli: &Cli, identifier: String) -> Result<()> {
    let mut config = crate::config::read_config(&cli.config)?;

    crate::config::remove_mod(&mut config, &identifier)?;
    crate::config::write_config(&cli.config, &config)?;

    println!("✓ Removed mod '{}' from configuration", identifier);
    Ok(())
}

async fn list_mods(cli: &Cli) -> Result<()> {
    let config = crate::config::read_config(&cli.config)?;

    if config.mods.is_empty() {
        println!("No mods configured. Use 'hsmm add' to add mods.");
        return Ok(());
    }

    println!("Configured mods ({}):\n", config.mods.len());

    for (i, mod_config) in config.mods.iter().enumerate() {
        let status = if mod_config.enabled { "✓" } else { "✗" };
        let id_str = match &mod_config.identifier {
            crate::config::ModIdentifier::ProjectId { curseforge } => format!("ID: {}", curseforge),
            crate::config::ModIdentifier::ProjectSlug { curseforge } => {
                format!("Slug: {}", curseforge)
            }
        };

        println!("{:3}. {} {} ({})", i + 1, status, mod_config.name, id_str);
    }

    Ok(())
}

async fn upgrade_mods(cli: &Cli) -> Result<()> {
    let mut config = crate::config::read_config(&cli.config)?;

    crate::upgrade::upgrade_mods(&mut config, &cli.output).await?;

    // Save updated config (game_id might have been auto-detected)
    crate::config::write_config(&cli.config, &config)?;

    Ok(())
}

async fn show_config(cli: &Cli) -> Result<()> {
    let config = crate::config::read_config(&cli.config)?;

    println!("Configuration:\n");
    println!("  Config file: {}", cli.config.display());
    println!("  Output directory: {}", cli.output.display());
    println!("  Game ID: {:?}", config.game_id);
    println!("  Mods: {}", config.mods.len());
    println!("\nSettings:");
    println!(
        "  Cleanup old versions: {}",
        config.settings.cleanup_old_versions
    );
    println!("  Max old versions: {}", config.settings.max_old_versions);
    println!(
        "  Download timeout: {}s",
        config.settings.download_timeout_secs
    );
    println!(
        "  Parallel downloads: {}",
        config.settings.parallel_downloads
    );

    Ok(())
}
