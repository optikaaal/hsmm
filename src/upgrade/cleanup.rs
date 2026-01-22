use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Clean up old mod versions
pub async fn cleanup_old_versions(
    mods_dir: &Path,
    current_files: &[String],
    settings: &crate::config::Settings,
) -> Result<()> {
    if !settings.cleanup_old_versions {
        tracing::info!("Old version cleanup is disabled");
        return Ok(());
    }

    tracing::info!("Cleaning up old mod versions...");

    let old_dir = mods_dir.join(".old");

    // Only create .old directory if archiving is enabled
    if settings.archive_old_versions {
        tokio::fs::create_dir_all(&old_dir)
            .await
            .with_context(|| format!("Failed to create .old directory: {}", old_dir.display()))?;
    }

    // Read all files in mods directory
    let mut entries = tokio::fs::read_dir(mods_dir)
        .await
        .with_context(|| format!("Failed to read mods directory: {}", mods_dir.display()))?;

    let mut moved_count = 0;
    let mut deleted_count = 0;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        // Skip directories and the .old directory itself
        if path.is_dir() {
            continue;
        }

        let file_name = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };

        // Delete .part files (incomplete downloads)
        if file_name.ends_with(".part") {
            tracing::debug!("Deleting incomplete download: {}", file_name);
            tokio::fs::remove_file(&path)
                .await
                .with_context(|| format!("Failed to delete .part file: {}", path.display()))?;
            deleted_count += 1;
            continue;
        }

        // Skip files that are in the current list
        if current_files.contains(&file_name) {
            continue;
        }

        // Archive or delete based on settings
        if settings.archive_old_versions {
            // Move old version to .old directory
            let old_path = old_dir.join(&file_name);
            tracing::debug!("Archiving old version: {} -> .old/", file_name);

            match tokio::fs::rename(&path, &old_path).await {
                Ok(_) => moved_count += 1,
                Err(e) => {
                    tracing::warn!(
                        "Failed to move {} to .old/, deleting instead: {}",
                        file_name,
                        e
                    );
                    tokio::fs::remove_file(&path).await.with_context(|| {
                        format!("Failed to delete old file: {}", path.display())
                    })?;
                    deleted_count += 1;
                }
            }
        } else {
            // Permanently delete old version
            tracing::debug!("Deleting old version: {}", file_name);
            tokio::fs::remove_file(&path)
                .await
                .with_context(|| format!("Failed to delete old file: {}", path.display()))?;
            deleted_count += 1;
        }
    }

    if moved_count > 0 || deleted_count > 0 {
        if settings.archive_old_versions {
            tracing::info!(
                "Cleaned up {} old versions (archived: {}, deleted: {})",
                moved_count + deleted_count,
                moved_count,
                deleted_count
            );
        } else {
            tracing::info!(
                "Cleaned up {} old versions (permanently deleted)",
                deleted_count
            );
        }
    } else {
        tracing::info!("No old versions to clean up");
    }

    // Cleanup old .old directory (keep only max_old_versions) - only if archiving
    if settings.archive_old_versions {
        cleanup_old_directory(&old_dir, settings.max_old_versions).await?;
    }

    Ok(())
}

/// Clean up old directory to keep only the most recent versions
async fn cleanup_old_directory(old_dir: &Path, max_versions: usize) -> Result<()> {
    if !old_dir.exists() {
        return Ok(());
    }

    let mut entries = tokio::fs::read_dir(old_dir)
        .await
        .with_context(|| format!("Failed to read .old directory: {}", old_dir.display()))?;

    // Collect all files with their modified times
    let mut files: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            if let Ok(metadata) = tokio::fs::metadata(&path).await {
                if let Ok(modified) = metadata.modified() {
                    files.push((path, modified));
                }
            }
        }
    }

    // Sort by modified time (newest first)
    files.sort_by(|a, b| b.1.cmp(&a.1));

    // Delete files beyond max_versions
    let mut deleted = 0;
    for (path, _) in files.iter().skip(max_versions) {
        tracing::debug!("Deleting old backup: {}", path.display());
        tokio::fs::remove_file(path)
            .await
            .with_context(|| format!("Failed to delete old backup: {}", path.display()))?;
        deleted += 1;
    }

    if deleted > 0 {
        tracing::info!("Deleted {} old backups from .old/", deleted);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cleanup_creates_old_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let settings = crate::config::Settings::default();

        cleanup_old_versions(temp_dir.path(), &[], &settings)
            .await
            .unwrap();

        assert!(temp_dir.path().join(".old").exists());
    }
}
