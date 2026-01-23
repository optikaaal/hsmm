pub mod cleanup;
pub mod download;

use anyhow::{Context, Result};
use std::path::Path;

use crate::config::{Config, ModIdentifier, ModMetadata};
use crate::curseforge;

/// Upgrade all mods to their latest versions
pub async fn upgrade_mods(config: &mut Config, mods_dir: &Path) -> Result<()> {
    tracing::info!("Starting mod upgrade process...");

    // Ensure we have the game ID
    if config.game_id.is_none() {
        tracing::info!("Game ID not set, detecting Hytale game ID...");
        let game_id = curseforge::get_hytale_game_id().await?;
        config.game_id = Some(game_id);
        tracing::info!("Using Hytale game ID: {}", game_id);
    }

    let game_id = config.game_id.unwrap();

    if config.mods.is_empty() {
        tracing::warn!("No mods configured. Use 'hsmm add' to add mods.");
        return Ok(());
    }

    // Filter enabled mods
    let enabled_mods: Vec<_> = config.mods.iter().filter(|m| m.enabled).collect();

    if enabled_mods.is_empty() {
        tracing::warn!("No enabled mods to update.");
        return Ok(());
    }

    tracing::info!("Checking {} mod(s) for updates...", enabled_mods.len());

    // Resolve mod identifiers to project IDs and fetch latest files
    let mut downloads = Vec::new();
    let mut current_files = Vec::new(); // Track all current files (downloaded + skipped)

    // Track which mods got which files for config updates
    let mut mod_to_file: Vec<(String, i32, String)> = Vec::new(); // (mod_name, project_id, filename)

    // Also track disabled mod files to prevent cleanup from removing them
    for mod_config in config.mods.iter().filter(|m| !m.enabled) {
        if let Some(ref filename) = mod_config.installed_file {
            current_files.push(filename.clone());
            tracing::debug!("Preserving disabled mod file: {}", filename);
        }
    }

    for mod_config in enabled_mods {
        match resolve_and_fetch_latest(&mod_config.name, &mod_config.identifier, game_id).await {
            Ok(metadata) => {
                let output_path = mods_dir.join(&metadata.file_name);

                // Add to current files list regardless of whether we download or skip
                current_files.push(metadata.file_name.clone());

                // Track this mod->file mapping
                mod_to_file.push((
                    mod_config.name.clone(),
                    metadata.project_id,
                    metadata.file_name.clone(),
                ));

                // Skip download if file already exists
                if output_path.exists() {
                    tracing::info!("  ✓ Already up to date: {}", metadata.file_name);
                    continue;
                }

                downloads.push((metadata, output_path));
            }
            Err(e) => {
                tracing::error!("Failed to fetch mod '{}': {}", mod_config.name, e);
            }
        }
    }

    if downloads.is_empty() {
        tracing::info!("All mods are up to date, no downloads needed.");
        // Still run cleanup to remove any old versions
        cleanup::cleanup_old_versions(mods_dir, &current_files, &config.settings).await?;
        tracing::info!("Mod upgrade complete!");
        return Ok(());
    }

    tracing::info!("Downloading {} mod(s)...", downloads.len());

    // Create mods directory if it doesn't exist
    tokio::fs::create_dir_all(mods_dir)
        .await
        .with_context(|| format!("Failed to create mods directory: {}", mods_dir.display()))?;

    // Prepare download list
    let download_list: Vec<_> = downloads
        .iter()
        .map(|(meta, path)| (meta.download_url.clone(), path.clone(), meta.file_length))
        .collect();

    // Download files in parallel
    let results = download::download_files_parallel(
        download_list,
        config.settings.download_timeout_secs,
        config.settings.parallel_downloads,
    )
    .await?;

    // Check for errors
    let mut success_count = 0;
    let mut error_count = 0;

    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(_) => {
                success_count += 1;
                // Update config with the installed filename
                let filename = &downloads[i].0.file_name;
                let project_id = downloads[i].0.project_id;
                if let Some(mod_config) = config
                    .mods
                    .iter_mut()
                    .find(|m| m.enabled && matches_mod(&m.identifier, project_id))
                {
                    mod_config.installed_file = Some(filename.clone());
                    tracing::debug!("Updated config: {} -> {}", mod_config.name, filename);
                }
            }
            Err(e) => {
                error_count += 1;
                tracing::error!("Failed to download {}: {}", downloads[i].0.file_name, e);
            }
        }
    }

    tracing::info!(
        "Download complete: {} succeeded, {} failed",
        success_count,
        error_count
    );

    // Update installed_file for ALL processed mods (both downloaded and already up-to-date)
    for (mod_name, project_id, filename) in mod_to_file {
        if let Some(mod_config) = config
            .mods
            .iter_mut()
            .find(|m| m.name == mod_name || matches_mod(&m.identifier, project_id))
        {
            mod_config.installed_file = Some(filename.clone());
            tracing::debug!("Tracked mod file: {} -> {}", mod_name, filename);
        }
    }

    // Clean up old versions (using current_files which includes both downloaded and skipped files)
    cleanup::cleanup_old_versions(mods_dir, &current_files, &config.settings).await?;

    tracing::info!("Mod upgrade complete!");

    Ok(())
}

/// Helper function to check if a mod identifier matches a project ID
fn matches_mod(identifier: &ModIdentifier, project_id: i32) -> bool {
    match identifier {
        ModIdentifier::ProjectId { curseforge } => *curseforge == project_id,
        _ => false,
    }
}

/// Resolve a mod identifier to a project ID and fetch the latest file
async fn resolve_and_fetch_latest(
    _name: &str,
    identifier: &ModIdentifier,
    game_id: i32,
) -> Result<ModMetadata> {
    // Resolve identifier to project ID
    let project_id = match identifier {
        ModIdentifier::ProjectId { curseforge } => *curseforge,
        ModIdentifier::ProjectSlug { curseforge } => {
            tracing::debug!("Resolving slug '{}' to project ID...", curseforge);
            curseforge::resolve_mod_slug(game_id, curseforge).await?
        }
    };

    // Fetch mod info to verify it exists and is valid
    let mod_info = curseforge::get_mod_info(game_id, project_id).await?;
    tracing::info!("Checking mod: {} (ID: {})", mod_info.name, project_id);

    // Get latest file
    let latest_file = curseforge::get_latest_mod_file(project_id).await?;
    tracing::info!(
        "  Latest version: {} ({})",
        latest_file.display_name,
        latest_file.file_name
    );

    // Convert to metadata
    curseforge::file_to_metadata(latest_file, project_id)
}
