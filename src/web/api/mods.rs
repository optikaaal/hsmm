use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::AppState;

#[derive(Debug, Serialize)]
pub struct ModInfo {
    pub name: String,
    pub enabled: bool,
    pub project_id: i32,
    pub installed: bool,
    pub version: Option<String>,
}

pub async fn list_mods(
    State(state): State<AppState>,
) -> Result<Json<Vec<ModInfo>>, (StatusCode, String)> {
    // Read mods.toml
    let config_content = tokio::fs::read_to_string(&state.config_path)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read config: {}", e),
            )
        })?;

    let config: crate::config::Config = toml::from_str(&config_content).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to parse config: {}", e),
        )
    })?;

    // Get list of installed mod files
    let installed_mods = match tokio::fs::read_dir(&state.mods_dir).await {
        Ok(mut entries) => {
            let mut mods = Vec::new();
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".jar") || name.ends_with(".zip") {
                        mods.push(name.to_string());
                    }
                }
            }
            mods
        }
        Err(_) => Vec::new(),
    };

    // Get CurseForge client to fetch actual filenames
    let api_key = std::env::var("CURSEFORGE_API_KEY").ok();
    let cf_client = api_key.as_ref().map(furse::Furse::new);

    let game_id = config.game_id.unwrap_or(0);

    let mut mod_infos = Vec::new();

    for m in config.mods {
        let project_id = match m.identifier {
            crate::config::ModIdentifier::ProjectId { curseforge } => curseforge,
            crate::config::ModIdentifier::ProjectSlug { .. } => 0,
        };

        // Try to get the actual filename from CurseForge API
        let expected_filename: Option<String> = if let Some(ref client) = cf_client {
            if project_id > 0 {
                // Fetch latest file for this mod
                match client.get_mod_files(project_id).await {
                    Ok(files) => {
                        // Filter and find the latest file for this game
                        let latest = files
                            .into_iter()
                            .filter(|f| f.game_id == game_id)
                            .max_by_key(|f| f.file_date);

                        latest.map(|f| f.file_name)
                    }
                    Err(_) => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        // Check if mod is installed
        let (installed, version) = {
            // First try: use the stored filename from config (most reliable!)
            if let Some(ref stored_filename) = m.installed_file {
                let file_exists = installed_mods.iter().any(|f| f == stored_filename);
                if file_exists {
                    (true, Some(stored_filename.clone()))
                } else {
                    // File was removed manually, clear it by falling through
                    (false, None)
                }
            } else {
                // Second try: exact filename match from API
                let exact_match = expected_filename
                    .as_ref()
                    .and_then(|filename| installed_mods.iter().find(|f| *f == filename).cloned());

                if exact_match.is_some() {
                    (true, exact_match)
                } else {
                    // Third try: fuzzy matching by mod name
                    // Extract key words from mod name for matching
                    let name_lower = m.name.to_lowercase();

                    let version = installed_mods
                        .iter()
                        .find(|f| {
                            let f_lower = f.to_lowercase();

                            // Strategy 1: Normalize and check if mod name is in filename
                            let f_normalized = f_lower
                                .replace(&[' ', '-', '_', '.', '\''][..], "")
                                .replace("macaws", "mcw")
                                .replace("hytale", "hy");
                            let name_normalized = name_lower
                                .replace(&[' ', '-', '_', '.', '\''][..], "")
                                .replace("macaw's", "mcw")
                                .replace("hytale", "hy");

                            if f_normalized.contains(&name_normalized)
                                || name_normalized.contains(&f_normalized)
                            {
                                return true;
                            }

                            // Strategy 2: Check for acronyms (e.g., "Just Enough Tales" -> "JET")
                            let acronym: String = name_lower
                                .split_whitespace()
                                .filter(|word| {
                                    !["and", "the", "a", "an", "of", "for"].contains(word)
                                })
                                .filter_map(|word| word.chars().next())
                                .collect();

                            if acronym.len() >= 2 && f_lower.contains(&acronym) {
                                return true;
                            }

                            // Strategy 3: Check individual significant words (3+ chars)
                            let significant_words: Vec<_> = name_lower
                                .split_whitespace()
                                .filter(|word| {
                                    word.len() >= 3 && !["and", "the", "mod"].contains(word)
                                })
                                .collect();

                            if !significant_words.is_empty() {
                                let matches = significant_words
                                    .iter()
                                    .filter(|word| f_lower.contains(*word))
                                    .count();

                                // If at least half the significant words match, consider it a match
                                if matches >= significant_words.len().div_ceil(2) {
                                    return true;
                                }
                            }

                            false
                        })
                        .cloned();

                    let installed = version.is_some();
                    (installed, version)
                }
            }
        };

        mod_infos.push(ModInfo {
            name: m.name,
            enabled: m.enabled,
            project_id,
            installed,
            version,
        });
    }

    Ok(Json(mod_infos))
}

#[derive(Debug, Deserialize)]
pub struct AddModRequest {
    pub name: String,
    pub project_id: i32,
}

pub async fn add_mod(
    State(state): State<AppState>,
    Json(req): Json<AddModRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Read current config
    let config_content = tokio::fs::read_to_string(&state.config_path)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read config: {}", e),
            )
        })?;

    let mut config: crate::config::Config = toml::from_str(&config_content).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to parse config: {}", e),
        )
    })?;

    // Check if mod already exists
    if config.mods.iter().any(|m| m.name == req.name) {
        return Err((
            StatusCode::CONFLICT,
            format!("Mod '{}' already exists", req.name),
        ));
    }

    // Add new mod
    config.mods.push(crate::config::Mod {
        name: req.name.clone(),
        enabled: true,
        identifier: crate::config::ModIdentifier::ProjectId {
            curseforge: req.project_id,
        },
        installed_file: None,
    });

    // Write back to config
    let new_content = toml::to_string_pretty(&config).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to serialize config: {}", e),
        )
    })?;

    tokio::fs::write(&state.config_path, new_content)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to write config: {}", e),
            )
        })?;

    Ok(Json(json!({
        "success": true,
        "message": format!("Mod '{}' added successfully", req.name)
    })))
}

#[derive(Debug, Deserialize)]
pub struct ToggleModRequest {
    pub enabled: bool,
}

pub async fn toggle_mod(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<ToggleModRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let config_content = tokio::fs::read_to_string(&state.config_path)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read config: {}", e),
            )
        })?;

    let mut config: crate::config::Config = toml::from_str(&config_content).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to parse config: {}", e),
        )
    })?;

    // Find and toggle mod
    let mod_found = config.mods.iter_mut().find(|m| m.name == name);

    match mod_found {
        Some(m) => {
            m.enabled = req.enabled;

            let new_content = toml::to_string_pretty(&config).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to serialize config: {}", e),
                )
            })?;

            tokio::fs::write(&state.config_path, new_content)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to write config: {}", e),
                    )
                })?;

            Ok(Json(json!({
                "success": true,
                "message": format!("Mod '{}' {} successfully", name, if req.enabled { "enabled" } else { "disabled" })
            })))
        }
        None => Err((StatusCode::NOT_FOUND, format!("Mod '{}' not found", name))),
    }
}

pub async fn remove_mod(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let config_content = tokio::fs::read_to_string(&state.config_path)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read config: {}", e),
            )
        })?;

    let mut config: crate::config::Config = toml::from_str(&config_content).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to parse config: {}", e),
        )
    })?;

    let initial_len = config.mods.len();
    config.mods.retain(|m| m.name != name);

    if config.mods.len() == initial_len {
        return Err((StatusCode::NOT_FOUND, format!("Mod '{}' not found", name)));
    }

    let new_content = toml::to_string_pretty(&config).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to serialize config: {}", e),
        )
    })?;

    tokio::fs::write(&state.config_path, new_content)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to write config: {}", e),
            )
        })?;

    Ok(Json(json!({
        "success": true,
        "message": format!("Mod '{}' removed successfully", name)
    })))
}
