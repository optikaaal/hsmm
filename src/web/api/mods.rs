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

    let mod_infos: Vec<ModInfo> = config
        .mods
        .into_iter()
        .map(|m| {
            let project_id = match m.identifier {
                crate::config::ModIdentifier::ProjectId { curseforge } => curseforge,
                crate::config::ModIdentifier::ProjectSlug { .. } => 0,
            };

            // Check if mod is installed by looking for files with the mod name
            let installed = installed_mods
                .iter()
                .any(|f| f.to_lowercase().contains(&m.name.to_lowercase()));

            let version = installed_mods
                .iter()
                .find(|f| f.to_lowercase().contains(&m.name.to_lowercase()))
                .cloned();

            ModInfo {
                name: m.name,
                enabled: m.enabled,
                project_id,
                installed,
                version,
            }
        })
        .collect();

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
