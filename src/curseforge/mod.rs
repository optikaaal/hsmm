use anyhow::{Context, Result};
use furse::Furse;
use std::sync::LazyLock;

/// Global CurseForge API client
pub static CURSEFORGE_API: LazyLock<Furse> = LazyLock::new(|| {
    // Default API key (same as ferium)
    const DEFAULT_KEY: &str = "$2a$10$sI.yRk4h4R49XYF94IIijOrO4i3W3dAFZ4ssOlNE10GYrDhc2j8K.";

    let api_key = std::env::var("CURSEFORGE_API_KEY")
        .ok()
        .filter(|key| !key.is_empty()) // Treat empty string as not set
        .unwrap_or_else(|| {
            tracing::debug!("Using default CurseForge API key");
            DEFAULT_KEY.to_string()
        });

    if api_key != DEFAULT_KEY {
        tracing::debug!("Using custom CurseForge API key from environment");
    }

    Furse::new(&api_key)
});

/// Get Hytale game ID
/// Returns the official CurseForge game ID for Hytale
pub async fn get_hytale_game_id() -> Result<i32> {
    // Check environment variable first (allows override for testing)
    if let Ok(game_id_str) = std::env::var("HYTALE_GAME_ID") {
        if let Ok(game_id) = game_id_str.parse::<i32>() {
            tracing::info!("Using Hytale game ID from environment: {}", game_id);
            return Ok(game_id);
        }
    }

    // Official Hytale game ID on CurseForge
    const HYTALE_GAME_ID: i32 = 70216;
    tracing::debug!("Using Hytale game ID: {}", HYTALE_GAME_ID);
    Ok(HYTALE_GAME_ID)
}

/// Get mod information by project ID
pub async fn get_mod_info(
    game_id: i32,
    project_id: i32,
) -> Result<furse::structures::mod_structs::Mod> {
    let mod_info = CURSEFORGE_API.get_mod(project_id).await.map_err(|e| {
        tracing::error!("CurseForge API error for mod {}: {:?}", project_id, e);
        anyhow::anyhow!(
            "Failed to fetch mod info for project ID {}: {}",
            project_id,
            e
        )
    })?;

    // Verify it's for the correct game
    if mod_info.game_id != game_id {
        anyhow::bail!(
            "Mod {} is for game ID {}, not Hytale (game ID {})",
            project_id,
            mod_info.game_id,
            game_id
        );
    }

    Ok(mod_info)
}

/// Resolve mod slug to project ID
/// Since furse doesn't have a search API, we'll just return an error with instructions
pub async fn resolve_mod_slug(_game_id: i32, slug: &str) -> Result<i32> {
    anyhow::bail!(
        "Cannot resolve mod slug '{}' automatically. Please use the CurseForge project ID instead. \
        Visit https://www.curseforge.com/hytale/mods/{} and use the project ID from the URL.",
        slug,
        slug
    )
}

/// Get the latest file for a mod
pub async fn get_latest_mod_file(project_id: i32) -> Result<furse::structures::file_structs::File> {
    tracing::debug!("Fetching files for mod {}", project_id);

    let mut files = CURSEFORGE_API
        .get_mod_files(project_id)
        .await
        .with_context(|| format!("Failed to fetch files for mod {}", project_id))?;

    // Sort by date (newest first)
    files.sort_by(|a, b| b.file_date.cmp(&a.file_date));

    // Get the first file (latest)
    files
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No files found for mod {}", project_id))
}

/// Convert CurseForge file to our metadata format
pub fn file_to_metadata(
    file: furse::structures::file_structs::File,
    project_id: i32,
) -> Result<crate::config::ModMetadata> {
    // Try to use the provided download_url, otherwise construct it from the file ID
    let download_url = if let Some(url) = file.download_url.clone() {
        url.to_string()
    } else {
        // Construct CurseForge CDN URL from file ID
        // Pattern: https://edge.forgecdn.net/files/{first4}/{last3}/{filename}
        // Example: file ID 7497127 -> https://edge.forgecdn.net/files/7497/127/filename.jar
        let file_id_str = file.id.to_string();
        if file_id_str.len() >= 4 {
            let first_part = &file_id_str[..file_id_str.len() - 3];
            let second_part = &file_id_str[file_id_str.len() - 3..];
            let constructed_url = format!(
                "https://edge.forgecdn.net/files/{}/{}/{}",
                first_part,
                second_part,
                urlencoding::encode(&file.file_name)
            );
            tracing::info!(
                "  ⚠ No download URL provided, constructed: {}",
                constructed_url
            );
            constructed_url
        } else {
            anyhow::bail!(
                "No download URL available and cannot construct URL for file {}",
                file.id
            );
        }
    };

    Ok(crate::config::ModMetadata {
        project_id,
        file_id: file.id,
        display_name: file.display_name,
        file_name: file.file_name,
        download_url,
        file_length: file.file_length as u64,
        file_date: file.file_date,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_get_hytale_game_id() {
        let game_id = get_hytale_game_id().await;
        assert!(game_id.is_ok(), "Should get Hytale game ID");
    }
}
